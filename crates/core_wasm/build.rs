use std::path::Path;
use std::process;

use swc_common::{FileName, GLOBALS, Mark, SourceMap, sync::Lrc};
use swc_ecma_codegen::{Config as EmitConfig, Emitter, text_writer::JsWriter};
use swc_ecma_minifier::optimize;
use swc_ecma_minifier::option::{CompressOptions, ExtraOptions, MangleOptions, MinifyOptions};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_transforms_typescript::{Config, typescript};
use swc_ecma_visit::swc_ecma_ast::{Pass, Program};

fn main() {
    println!("cargo:rerun-if-changed=src/helper.ts");
    println!("cargo:rerun-if-changed=src/shader.wgsl");
    println!("cargo:rerun-if-changed=tsconfig.json");

    let helper_path = Path::new("src/helper.ts");
    let shader_path = Path::new("src/shader.wgsl");

    if !helper_path.exists() || !shader_path.exists() {
        eprintln!("error: helper.ts or shader.wgsl not found");
        process::exit(1);
    }

    let helper_source = std::fs::read_to_string(helper_path).unwrap();
    let shader_source = std::fs::read_to_string(shader_path).unwrap();

   // Validate original shader with naga (catches author errors)
    validate_wgsl(&shader_source, "original shader");

    // Minify WGSL shader before embedding
    let minified_wgsl = minify_wgsl(&shader_source);

    // Validate minified shader with naga (catches minifier bugs)
    validate_wgsl(&minified_wgsl, "minified shader");

    // Escape WGSL for embedding in a JS string literal
    let escaped_wgsl = minified_wgsl
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
        .replace("`", "\\`")
        .replace('\r', "");

    // Embed the WGSL shader into the helper source
    let combined = helper_source.replace("$$<WGSL_PLACEHOLDER>$$", &escaped_wgsl);

    // Parse as TypeScript, strip types, minify with SWC
    let minified = GLOBALS.set(&Default::default(), || {
        let cm: Lrc<SourceMap> = Default::default();

        // --- Step 1: Parse TypeScript ---
        let fm = cm.new_source_file(
            FileName::Custom("helper.ts".into()).into(),
            combined.clone(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().expect("Failed to parse TypeScript");

        // --- Step 2: Strip TypeScript types ---
        let unresolved_mark = Mark::fresh(Mark::root());
        let top_level_mark = Mark::fresh(Mark::root());
        let config = load_tsconfig();
        let mut stripper = typescript(config, unresolved_mark, top_level_mark);
        let mut program = Program::Module(module);
        stripper.process(&mut program);

        // --- Step 3: Minify with SWC ---
 let minify_options = MinifyOptions {
            compress: Some(CompressOptions {
                dead_code: true,
                drop_console: true,
                drop_debugger: true,
                evaluate: true,
                side_effects: true,
                unused: true,
                sequences: 3,
                reduce_vars: true,
                reduce_fns: true,
                collapse_vars: true,
                comparisons: true,
                conditionals: true,
                directives: true,
                hoist_props: true,
                if_return: true,
                inline: 3,
                join_vars: true,
                loops: true,
                merge_imports: true,
                negate_iife: true,
                props: true,
                typeofs: true,
                module: false,
                ..Default::default()
            }),
            mangle: Some(MangleOptions {
                top_level: Some(false),
                keep_class_names: false,
                keep_fn_names: false,
                ..Default::default()
            }),
            ..Default::default()
        };

        let extra_options = ExtraOptions {
            unresolved_mark,
            top_level_mark,
            mangle_name_cache: None,
        };

        let optimized = optimize(
            program,
            cm.clone(),
            None,
            None,
            &minify_options,
            &extra_options,
        );

        // --- Step 4: Emit minified JS ---
        let mut output = Vec::new();
 let mut emitter = Emitter {
            cfg: EmitConfig::default()
                .with_minify(true)
                .with_ascii_only(true)
                .with_omit_last_semi(true),
            comments: None,
            cm: cm.clone(),
            wr: JsWriter::new(cm, "\n", &mut output, None),
        };

        emitter
            .emit_program(&optimized)
            .expect("Failed to emit minified JS");

        String::from_utf8(output).expect("JS output is not valid UTF-8")
    });

    // Write to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("helper.min.js");
    std::fs::write(&dest_path, &minified).expect("Failed to write minified JS");

    println!(
        "cargo:warning=WGSL shader: {} -> {} bytes, full JS: {} -> {} bytes",
        shader_source.len(),
        minified_wgsl.len(),
        combined.len(),
        minified.len()
    );
}

/// Minify WGSL shader source: strip comments, collapse whitespace,
/// and compact struct field alignment spacing.
///
/// Identifier mangling is intentionally skipped — WGSL has many semantically
/// significant names (built-in attrs, entry points, struct fields, global vars)
/// that cannot be safely renamed without a full parser.
fn minify_wgsl(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(source.len() / 2);
    let mut i = 0;
    let mut prev = None;

    while i < len {
        let c = chars[i];

        // Single-line comment //
        if c == '/' && i + 1 < len && chars[i + 1] == '/' && !in_string(&chars, i) {
            i += 2;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            prev = None;
            continue;
        }

        // Multi-line comment /* */
        if c == '/' && i + 1 < len && chars[i + 1] == '*' && !in_string(&chars, i) {
            i += 2;
            while i + 1 < len {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            prev = None;
            continue;
        }

        // Whitespace collapse: replace runs of whitespace with single space
        if c.is_whitespace() {
            i += 1;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            // Only emit space if both sides need separation
            if prev.is_some() && i < len && needs_sep(chars[i]) {
                out.push(' ');
                prev = Some(' ');
            }
            continue;
        }

        // Remove space around colons (struct field alignment)
        if c == ':' {
            // strip trailing space before colon
            while out.ends_with(' ') {
                out.pop();
            }
            out.push(c);
            prev = Some(c);
            i += 1;
            // strip leading space after colon
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            // add space after colon if next char needs separation
            if i < len && needs_sep(chars[i]) {
                out.push(' ');
            }
            continue;
        }

        // Space around dots for member access: vec3<f32>(x, y, z) vs p.r
        // No extra space needed around dots, parens, braces
        out.push(c);
        prev = Some(c);
        i += 1;
    }

    out.trim_end().to_string()
}

/// Check if character needs a separator from the preceding token
fn needs_sep(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.' || c == '/'
}

/// Check if position `i` is inside a string literal by scanning backwards for unescaped quotes
fn in_string(chars: &[char], i: usize) -> bool {
    let mut in_str = false;
    let mut j = 0;
    while j < i {
        if chars[j] == '"' {
            in_str = !in_str;
        } else if chars[j] == '\\' {
            j += 1; // skip escape
        }
        j += 1;
    }
    in_str
}

/// Validate WGSL source by parsing with naga — catches syntax and type errors.
/// Full validation is skipped because naga is stricter than the WebGPU runtime.
fn validate_wgsl(source: &str, label: &str) {
    // Strip @entry_point("...") attributes — naga doesn't support them yet
    let sanitized = strip_entry_point_attrs(source);

    let module = naga::front::wgsl::parse_str(&sanitized).unwrap_or_else(|e| {
        eprintln!("error: failed to parse {label}:\n{e}");
        process::exit(1);
    });

  println!(
        "cargo:warning={label}: {} types, {} functions, {} global vars, {} entry points",
        module.types.len(),
        module.functions.len(),
        module.global_variables.len(),
        module.entry_points.len(),
    );
}

/// Strip @entry_point("...") attributes not yet supported by naga's WGSL parser.
fn strip_entry_point_attrs(s: &str) -> String {
    const MARKER: &str = "@entry_point(\"";
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + MARKER.len() <= chars.len() && chars[i..i + MARKER.len()].iter().collect::<String>() == MARKER {
            i += MARKER.len();
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing "
            }
            if i < chars.len() && chars[i] == ')' {
                i += 1; // skip closing )
            }
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Load tsconfig.json and map supported fields to SWC's TypeScript transformer Config.
///
/// SWC's `Config` struct maps a small subset of tsconfig compilerOptions:
/// - `verbatimModuleSyntax`
/// - `importNotUsedAsValues`
/// - `nativeClassProperties` / `useDefineForClassFields`
/// - `noEmptyExport`
/// - `tsEnumIsMutable`
/// - `flow`
///
/// Most tsconfig fields (`target`, `module`, `lib`, `strict`, `types`, `noEmit`) are
/// TypeScript-compiler concepts that don't apply to SWC's type-stripping pipeline.
/// SWC strips types purely syntactically — no type resolution is needed.
fn load_tsconfig() -> Config {
    let tsconfig_path = Path::new("tsconfig.json");
    if !tsconfig_path.exists() {
        return Config::default();
    }

    let Ok(content) = std::fs::read_to_string(tsconfig_path) else {
        return Config::default();
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Config::default();
    };

    let Some(options) = value.get("compilerOptions").and_then(|v| v.as_object()) else {
        return Config::default();
    };

    let mut config = Config::default();

    if let Some(true) = options.get("verbatimModuleSyntax").and_then(|v| v.as_bool()) {
        config.verbatim_module_syntax = true;
    }

    if let Some(val) = options.get("importNotUsedAsValues").and_then(|v| v.as_str()) {
        config.import_not_used_as_values = match val {
            "preserve" => swc_ecma_transforms_typescript::ImportsNotUsedAsValues::Preserve,
            _ => swc_ecma_transforms_typescript::ImportsNotUsedAsValues::Remove,
        };
    }

    if let Some(true) = options.get("nativeClassProperties").and_then(|v| v.as_bool()) {
        config.native_class_properties = true;
    }

    if let Some(false) = options.get("useDefineForClassFields").and_then(|v| v.as_bool()) {
        config.native_class_properties = false;
    }

    if let Some(true) = options.get("noEmptyExport").and_then(|v| v.as_bool()) {
        config.no_empty_export = true;
    }

    if let Some(true) = options.get("tsEnumIsMutable").and_then(|v| v.as_bool()) {
        config.ts_enum_is_mutable = true;
    }

    if let Some(true) = options.get("flow").and_then(|v| v.as_bool()) {
        config.flow_syntax = true;
    }

    config
}
