import sys
import os
from . import (
    parse_json,
    parse_json_file,
    dumps,
    generate_json_with_size,
    generate_nested_json,
    benchmark_performance,
)


def compare_to_rust(rust_time, other_time):
    """Calculates if Rust is faster or slower and returns the formatted string."""
    # Safety check for zero division
    rust_time = max(rust_time, 1e-12)
    other_time = max(other_time, 1e-12)

    if rust_time <= other_time:
        # Rust is faster (or equal)
        speedup = other_time / rust_time
        return f"(Rust is {speedup:.2f}x faster)"
    else:
        # Rust is slower
        slowdown = rust_time / other_time
        return f"(Rust is {slowdown:.2f}x slower)"


def run_benchmark_suite():
    """Runs a series of benchmarks and prints a formatted report."""
    print("🚀 Starting Rust JSON Parser Benchmark Suite...")
    print("Reporting Median Values (Warmup included)\n")

    scenarios = [
        ("Small JSON", generate_json_with_size(30), 2000),
        ("Medium JSON", generate_json_with_size(10240), 1000),
        ("Large JSON", generate_json_with_size(102400), 500),
        ("Deeply Nested", generate_nested_json(50), 1000),
    ]

    for label, data, iters in scenarios:
        size_bytes = len(data.encode('utf-8'))
        # Update the header to show the specific iteration count for this test
        print(f"--- {label} ({size_bytes} bytes) | {iters} iterations ---")

        try:
            rust_t, py_t, simple_t = benchmark_performance(data, iterations=iters)

            rust_t = max(rust_t, 1e-9)
            json_speedup = py_t / rust_t
            simple_speedup = simple_t / rust_t

            print(f"  Rust:             {rust_t:.6f}s")
            print(f"  Python json (C):  {py_t:.6f}s  {compare_to_rust(rust_t, py_t)}")
            print(f"  simplejson:       {simple_t:.6f}s  {compare_to_rust(rust_t, simple_t)}")
            print()
        except Exception as e:
            print(f"  ❌ Benchmark failed for {label}: {e}\n")


def main():
    # Handle the Benchmark Flag
    if "--benchmark" in sys.argv:
        run_benchmark_suite()
        sys.exit(0)
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
