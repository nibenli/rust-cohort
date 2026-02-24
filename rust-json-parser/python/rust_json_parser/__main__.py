import sys
import os
from . import parse_json, parse_json_file, dumps


def main():
    # Handle Input (File, String, or Stdin)
    if len(sys.argv) < 2:
        # Check if data is being piped in via stdin
        if not sys.stdin.isatty():
            input_data = sys.stdin.read()
            try:
                processed_obj = parse_json(input_data)
            except Exception as e:
                print(f"Error parsing stdin: {e}", file=sys.stderr)
                sys.exit(1)
        else:
            print("Usage: python -m rust_json_parser <json_string_or_file_path>")
            sys.exit(1)
    else:
        target = sys.argv[1]

        if target.endswith('.json') and not os.path.exists(target):
            print(f"Error: File not found: {target}", file=sys.stderr)
            sys.exit(1)

        try:
            # Decide between File or Json String
            if os.path.isfile(target):
                processed_obj = parse_json_file(target)
            else:
                processed_obj = parse_json(target)
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            sys.exit(1)

    # Pretty-print the result back to the console
    # Using Rust 'dumps' function with indentation
    print(dumps(processed_obj, indent=2))


if __name__ == "__main__":
    main()
