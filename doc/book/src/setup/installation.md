# Installation

RHDL functions primarily as a Rust crate.  Installation steps to get up and running are:

1. Install Rust and Cargo.  Follow the instructions at [rustup.rs](https://rustup.rs/) to install the Rust toolchain.  RHDL is not a good first Rust project, so make sure you have some familiarity with Rust before proceeding.
2. Install Icarus Verilog (optional).  You can use Icarus Verilog to simulate RHDL-generated Verilog code.  Follow the instructions on the [Icarus Verilog Website](https://steveicarus.github.io/iverilog/usage/installation.html) to install Icarus Verilog on your system.
3. Install [Surfer](https://surfer-project.org/) (optional).  Installation instructions are [here](https://docs.surfer-project.org/book/#installation).
4. Install the `rhdl-surfer-plugin` (optional).  This plugin allows Surfer to read RHDL trace files and provides type information and decoding in the waveform viewer.  It is highly recommended.  The plugin will be available somewhere.
5. Install your FPGA toolchain (optional).  If you plan to synthesize RHDL designs for an FPGA, install the appropriate toolchain for your target FPGA.  For example, for Xilinx FPGAs, you can install Vivado or Vitis.  Follow the instructions provided by the FPGA vendor for installation.  You can also install an open source toolchain if one exists for your target FPGA.
6. Install VSCode (optional).  I recommend using Visual Studio Code as your IDE for RHDL development.  You can download it from [here](https://code.visualstudio.com/).
7. Install the Rust Analyzer extension for VSCode (optional).  This extension provides enhanced Rust support in VSCode, including code completion, error checking, and more.  You can find it in the VSCode extensions marketplace by searching for "rust-analyzer".

While steps 2-7 are optional, they are all recommended for a better development experience with RHDL.