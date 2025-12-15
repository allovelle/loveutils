r"""
typed_os_pipe - concept to see if this would be more convenient than
    defaulting to raw text all the time.

usage:
    write-stdou
        ||           read-stdin & write-stdou
        ||                      ||           read-stdin & print
        ||                      ||                      ||
        \/                      \/                      \/
        py ./typed_os_pipe.py | py ./typed_os_pipe.py | py ./typed_os_pipe.py

    write-stdou
        ||           read-stdin & print
        ||                      ||
        \/                      \/
        py ./typed_os_pipe.py | py ./typed_os_pipe.py

    show-help
        ||
        \/
        py ./typed_os_pipe.py
"""

import sys, pickle


def main():
    obj = dict(
        name="Alo",
        age=30,
        addr=("San Carlos St\n" "Carmel, CA 93921\n" "United States").strip(),
        call=1j + 8316261111,
    )

    stdin_piped = not sys.stdin.isatty()
    stdout_piped = not sys.stdout.isatty()

    # TODO: Copy this to PQ
    if not stdin_piped and stdout_piped:  # SELF is begin | A
        print("Start of pipe chain: THIS | other", file=sys.stderr)
        pickle.dump(obj, file=sys.stdout.buffer)

    elif stdin_piped and stdout_piped:  # A | SELF is mid | B
        print("Middle of pipe chain: other | THIS | other", file=sys.stderr)
        try:
            obj = pickle.load(file=sys.stdin.buffer)
        except pickle.UnpicklingError:
            sys.exit(print("stdin sent non-pickled data"))
        pickle.dump(obj, file=sys.stdout.buffer)

    elif stdin_piped and not stdout_piped:  # A | SELF is end
        print("End of pipe chain: other | THIS", file=sys.stderr)
        try:
            obj = pickle.load(file=sys.stdin.buffer)
        except pickle.UnpicklingError:
            sys.exit(print("stdin sent non-pickled data"))
        except EOFError:
            sys.exit(print("unexpected eof from stdin"))
        lit = pickle.dumps(obj)
        print(f"\nlit={lit!r}")
        print(f"\n{obj=}")

    else:  # SELF show help message
        print("Standalone execution: THIS", file=sys.stderr)
        sys.exit(print(__doc__))


if __name__ == "__main__":
    main()
