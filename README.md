ttyGraph
==========
[![Build Status](https://travis-ci.org/makutamoto/ttygraph.svg?branch=master)](https://travis-ci.org/makutamoto/ttygraph)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A graph viewer for character user interface.

Installation
----------
You can download pre-compiled binary file from below, or choose to build from source code.

Linux: [ttygraph-linux-latest.tar.gz](./releases/ttygraph-linux-latest.tar.gz)  
macOS: [ttygraph-macos-latest.zip](./releases/ttygraph-macos-latest.zip)  

### Build from source code
#### Prerequisites
This program is written in Rust. So you need to install Rust and Cargo by reading [rust-lang.org](https://www.rust-lang.org/tools/install).

#### Build  
Clone this repository to your computer by running:  
`git clone https://github.com/makutamoto/ttygraph.git`

In the cloned directory, run:  
`cargo run`

Usage
----------
![Initial screen](./resources/usage.png)

The center positioned `#` is a cursor. You can select a graph by overlapping with this.

* `arrow keys` : move around the coordinate plane
* `ctrl` + `X` : quit this program
* `ctrl` + `A` : add a new graph
* `ctrl` + `E` : edit a selected graph
* `ctrl` + `D` : delete a selected graph
* `ctrl` + `C` : back to center (origin)

### Inputting a formula
![editing](./resources/inputting.png)

This program use mathematical formulae to display graphs such as `x ^ 2 + y ^ 2 = 100` as shown in the above image.

Formulae must conform to this syntax: `[left] = [right]`

#### Supported Operators
* `^` : Power
* `*` : Multiplication
* `/` : Division
* `%` : Modulus
* `+` : Addition
* `-` : Subtraction

#### Supported Functions
* `abs(A)` : Returns the absolute value of *A*.
* `max(A, B)` : Returns the maximum of *A* and *B*.
* `min(A, B)` : Returns the minimum of *A* and *B*.
* `ln(A)` : Calculates ln(*A*).
* `log(A, B)`: Calculates the logarithm of *B* with *A* as the base.
* `log2(A)` : Calculates the logarithm of *A* with 2 as the base.
* `log10(A)` : Calculates the common logarithm of *A*.
* `root(A, B)` : Calculates the *A*-th root of *B*.
* `sqrt(A)` : Calculates the square root of *A*.
* `cbrt(A)`: Calculates the cubic root of *A*.
* `sin(A)` : Calculates the sin of *A*.
* `cos(A)` : Calculates the cos of *A*.
* `tan(A)` : Calculates the tan of *A*.
* `asin(A)` : Calculates the asin of *A*.
* `acos(A)` : Calculates the acos of *A*.
* `atan(A)` : Calculates the atan of *A*.
* `sinh(A)` : Calculates the sinh of *A*.
* `cosh(A)` : Calculates the cosh of *A*.
* `tanh(A)` : Calculates the tanh of *A*.
* `asinh(A)` : Calculates the asinh of *A*.
* `acosh(A)` : Calculates the acosh of *A*.
* `atanh(A)` : Calculates the atanh of *A*.
* `ceil(A)` : Calculates the ceil of *A*.
* `floor(A)` : Calculates the floor of *A*.
* `round(A)` : Rounds *A* to an integer.

#### Constants
* `PI` : The ratio of the circumference of a circle. (3.14...)
* `e` : Euler's number. (2.71...)

License
----------
ttyGraph is released under GPLv3.0.
