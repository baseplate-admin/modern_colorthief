use std::path::Path;
use std::process;

use swc_common::{FileName, Mark, SourceMap, GLOBALS, sync::Lrc};
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::swc_ecma_ast::{Pass, Program};

fn main() {
    println!("cargo:rerun-if-changed=src/helper.ts");
    println!("cargo:rerun-if-changed=src/shader.wgsl");

    let helper_path = Path::new("src/helper.ts");
    let shader_path = Path::new("src/shader.wgsl");

    if !helper_path.exists() || !shader_path.exists() {
        eprintln!("error: helper.ts or shader.wgsl not found");
        process::exit(1);
    }

    let helper_source = std::fs::read_to_string(helper_path).unwrap();
    let shader_source = std::fs::read_to_string(shader_path).unwrap();

    // Escape WGSL for embedding in a JS string literal
    let escaped_wgsl = shader_source
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
        .replace("`", "\\`")
        .replace('\r', "");

    // Embed the WGSL shader into the helper source
    let combined = helper_source.replace("$$<WGSL_PLACEHOLDER>$$", &escaped_wgsl);

    // Parse as TypeScript, strip types, emit as JS (within SWC global context)
    let js = GLOBALS.set(&Default::default(), || ts_to_js(&combined));

    // Write to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("helper.min.js");
    std::fs::write(&dest_path, &js).expect("Failed to write helper JS");

    println!(
        "cargo:warning=Embedded WGSL shader: {} -> {} bytes",
        combined.len(),
        js.len()
    );
}

fn ts_to_js(source: &str) -> String {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(
        FileName::Custom("helper.ts".into()).into(),
        source.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().expect("Failed to parse TypeScript");

    let unresolved_mark = Mark::fresh(Mark::root());
    let top_level_mark = Mark::fresh(Mark::root());
    // Create the TypeScript-stripping transform and apply it
    let mut stripper = strip(unresolved_mark, top_level_mark);
    let mut program = Program::Module(module);
    stripper.process(&mut program);

    let mut output = Vec::new();

    {
        let cm = cm.clone();
        let mut emitter = Emitter {
            cfg: Default::default(),
            comments: None,
            cm: cm.clone(),
            wr: JsWriter::new(cm, "\n", &mut output, None),
        };

        if let Program::Module(ref mod_) = program {
            emitter.emit_module(mod_).expect("Failed to emit JS");
        }
    }

    String::from_utf8(output).expect("JS output is not valid UTF-8")
}
