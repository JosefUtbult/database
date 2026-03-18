#!/bin/bash

cargo expand --example macro_test --features debug_macro > examples/debug_macro_expanded.rs
