Tom Kerrigan's Simple Chess Program (TSCP)

Copyright 2016 Tom Kerrigan

(Rust port by Kristopher Johnson)

## README

TSCP is intended for people who want to learn about chess programming. Its
source code is designed to be very easy to understand.

The TSCP "package" includes these files:

- README.md - The file you're reading now!
- book.txt - TSCP's opening library, i.e., common sequences of initial moves.
- Cargo.toml - The manifest for the Rust Cargo package manager
- src/*.rs - The Rust source code for TSCP

To use TSCP, you have to compile the source code.  For information on installing
Rust and Cargo, see <https://www.rust-lang.org/tools/install>.  When the Rust
tools are installed, you can build the program by running this command:

    cargo build

This will build the `tscp` executable in the `target/debug` directory.  (For a
non-debug build, run `cargo build --release` and the executable will be in the
`target/release` directory.)

To build and run the program in a single step, run this command:

    cargo run

Once the program is running, type "help" at the prompt.

TSCP works with Tim Mann's graphical user interface XBoard/WinBoard. For more
information about Tim's program, visit his chess web page:
http://www.research.digital.com/SRC/personal/Tim_Mann/chess.html
(Note that TSCP doesn't support every WinBoard feature.)

If you're studying TSCP's source code, the following files will be
particularly helpful:

- defs.rs - This has all of TSCP's macros and data types.
- data.rs - This is where all of TSCP's global data is declared, and it's fairly well commented.

If you have any questions, comments, suggestions, etc., please feel free to
contact me:

- E-mail: tom.kerrigan@gmail.com
- WWW: http://www.tckerrigan.com/

I hope you find TSCP useful!

-Tom

## LEGAL STUFF

According to copyright law, you are not allowed to distribute copies of TSCP
or anything that's derived from TSCP without my authorization.

Version 1.4 of TSCP is the first version to include copyright notices, but
previous versions are also protected under law. If you are distributing an
earlier version of TSCP or a derivative work without my authorization, you are
acting illegally.

For more information about copyrights, visit this web page:
http://lcweb.loc.gov/copyright/

## THANKS

The following people have helped me with TSCP:

- Normand Blais
- Daniel Karlsson
- Ruben Razquin
- Brian Richardson
- William Bryant
- Mathias Feist
- Shinya Kotani
- John Stanback
- David Brownlee
- Uri Blass
- Dann Corbit
- Pedro Castro Elgarresta
- Kristopher Johnson

## VERSION HISTORY

Version 1.81c, 1/3/19

* Fixed the makefile so it compiles the .c files with optimizations.

Version 1.81b, 3/10/16

* Small revisions to this README file; updated my e-mail address and home page URL.
* Separated TSCP's source code and the Windows executable.
* Added a makefile.
* Updated parse_move in main.c to allow lower-case letters for pawn promotions.
* print_result claimed a threefold repetition draw after a position occurred four times instead of three. Fixed.

Version 1.81, 2/5/03

* fflush() wasn't being called after print_results() in xboard().
* hply was being set to 0 when xboard() received "black." This broke the opening book code when playing black with WinBoard.
* set_hash() wasn't being called in bench().

Version 1.8, 1/30/03

* Added opening book code (book.c) and a small opening book (book.txt).
* Added hash keys; replaced the old repetition detection code with code that compares hash keys.
* Changed bench to report a "Score" to avoid confusion/comparisons with the previous version's "MIPS" (with score, 1.000 = my Athlon XP 2000+).
* Moved move parsing into its own function (parse_move in main.c).

Version 1.73, 2/11/01

* Added code to handle EOFs from stdin.

Version 1.72, 2/1/01

* Added code to stop searching if a mate is found.
* Changed max_time/max_depth assignments in main.c.

Version 1.71, 10/30/00

* bench() now reports MIPS.
* main() now returns an int.
* Added an fflush() to think().
* Changed %ld format strings to %d.

Version 1.7, 10/28/00

* New (much cleaner) king safety.
* Removed pawn_count because it was redundant.
* Replaced gen_begin and gen_end with first_move.
* Minor speedup of in_check().
* Added print_board() to bench().
* Added a new test to make sure ftime() returns milliseconds.
* Changed GetRank/GetFile macros to ROW/COL.
* Changed longs back to ints so I don't have to keep track of which is which.

Version 1.6, 7/17/00

* Switched "game history" method from undo_dat to hply.
* Changed the names of some macros.
* Added repetition detection.
* Added support for XBoard post/nopost commands (search output).
* Added print_result() to print the result of games.

Version 1.52, 6/29/00

* Fixed a typo in eval_dark_king(); a DARK should have been LIGHT.

Version 1.51, 6/16/00

* Added support for the XBoard "time" command.
* Changed max_time from seconds to milliseconds.
* Changed print_move() to move_str().

Version 1.5, 2/13/00

* Changed all of the "time control" code. Now TSCP can search for a fixed amount of time.
* Stopped searching all responses to check in the quiescence search--it was causing the search tree to blow up.
* Fixed passed pawn scoring asymmetry.
* Added a test to make sure the search doesn't go too deep and overflow everything.

Version 1.42, 12/12/99

* Made "xboard" a command instead of an argument.
* Fixed code to start a new game.
* Added a test to make sure ftime() is returning milliseconds.
* Fixed king safety bug; now checks all three files around an uncastled king.
* Changed gets() to fgets() in xboard() to avoid a gcc warning.
* Added fflush() after the illegal XBoard command message.
* Added signal(SIGINT, SIG_IGN) to the beginning of xboard() to help with XBoard/Linux compatibility.

Version 1.41, 12/6/99

* The bonus for black passed pawns was backwards.

Version 1.4, 12/6/99

* Added the ability to undo moves.
* Added XBoard/WinBoard support.
* Added the benchmarking feature.
* Added the skill command.
* Wrote the README file.
* Added a Win32 executable to the "distribution."
* Changed main() so the engine doesn't play an illegal move if it's mated.
* Changed some array sizes and some ints to longs so TSCP works correctly with 16-bit compilers.
* Got rid of warnings. TSCP now compiles without warnings using Visual C++, gcc, and Turbo C 2.
* Added whitespace in the source code to make it more readable. "x=2;" is now "x = 2;", etc.
* Made a small number of minor (mostly aesthetic) changes to the algorithms.
* Added copyright notices to the beginning of each source file and the text that TSCP prints when you run it.

## RUST PORT

The original C code was translated to Rust by Kristopher Johnson. The
translation is a derived work, and copyright belongs to Tom Kerrigan.

In general, the C code has been translated into Rust in the most straightforward
way.  The module structure, identifier names, and logic of the Rust code are the
same as those in the original C code, without any re-design to be more idiomatic
as Rust.  So, this source code is primarily useful for people who want to learn
about chess programming, and not for people who want to learn how to design Rust
programs.

Rust-specific comments start with the tag `#rust`.  All other comments are
copied from Tom Kerrigan's original C code, with minor edits as needed to match
the Rust translation.

These stylistic rules have been applied during the translation:

- The Rust code follows the naming conventions specified in [RFC #430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md).  Static variables are in `SCREAMING_SNAKE_CASE` and type names are in `UpperCamelCase`.  So, for example, the C global variable `gen_dat` has been renamed `GEN_DAT` in the Rust code, and the C `struct move_bytes` type has been renamed `struct MoveBytes` in Rust.
- Comments have been reformatted as Rust-style comments.
- The `rustfmt` tool is used to automatically reformat Rust code in a consistent style.
- This README file was converted from plain ASCII text to Markdown format, for easier reading on GitHub.
