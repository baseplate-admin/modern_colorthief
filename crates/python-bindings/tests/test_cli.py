import sys
from unittest.mock import patch
import pytest
from modern_colorthief.cli import main
from pathlib import Path
import os

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = os.path.join(BASE_DIR, "kaiju_no_8.jpg")


def test_cli_dominant_color(capsys):
    """Test the CLI for getting a dominant color."""
    test_args = ["modern_colorthief", TEST_IMAGE]
    with patch.object(sys, "argv", test_args):
        main()

    captured = capsys.readouterr()
    # Expect a single hex string (e.g., #rrggbb) followed by a newline
    output = captured.out.strip()
    assert output.startswith("#")
    assert len(output) == 7


def test_cli_palette(capsys):
    """Test the CLI for getting a palette."""
    count = 3
    test_args = ["modern_colorthief", TEST_IMAGE, "--palette", "--count", str(count)]
    with patch.object(sys, "argv", test_args):
        main()

    captured = capsys.readouterr()
    lines = captured.out.strip().split("\n")
    # Depending on implementation, there could be fewer colors if image has few colors,
    # but kaiju_no_8.jpg seems to have plenty.
    # We just check we got some output formatted correctly.
    assert len(lines) == count
    for line in lines:
        assert line.startswith("#")
        assert len(line.strip()) == 7


def test_cli_invalid_file(capsys):
    """Test the CLI handles invalid files gracefully."""
    test_args = ["modern_colorthief", "non_existent_file.jpg"]
    with patch.object(sys, "argv", test_args):
        with pytest.raises(SystemExit) as e:
            main()

    assert e.value.code == 1
    captured = capsys.readouterr()
    assert "Error processing image" in captured.err
