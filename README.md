# Project RustySpark - Master thesis of a game engineering student

## Whats is RustySpark?

RustySpark is the Rust counterpart to Spark, a C++ implementation of commonly used game engine submodules. RustySpark will
implement the same modules as the C++ Spark project in order to compare the performance of both implementations. On one hand
RustySpark shall show how performant game engine submodules can be implemented in stable Rust and on the other hand it is the
base to measure and compare the runtime performance to the C++ counterparts.

## What will be implemented?

The sub-modules I choose are:

- Memory Management (MemoryArenas providing different allocators and debug capabilities)
- Containers (commonly used at runtime/for tooling, Vector, HashMap, Fixed-sized array, trees, ...)
- Entity Component System (ECS and systems to update/create components)

## What is contained in this repository?

This repo contains the Rust implementation of project Spark - called RustySpark. It will include the three submodules, compiled as
static libraries, unit-tests and a benchmark application to run benachmarks and measurements. 

## What is not included in this repository?

This repository does not include the other part of project Spark - Spark++. Spark++ is the C++ implementation of the sub-modules which is hosted in a separate repository. Spark++ uses premake as build system and also includes the three module projects as static libraries, a benchmark application as well as unit-tests handled by the google-test unit-test framework.

## Where can I find the results?

As soon as the results are ready a third, general project Spark repository will be created that contains both, Spark++ and RustySpark, as submodules. Beside the projects it will feature the Master's thesis and all measurements and findings of the process.