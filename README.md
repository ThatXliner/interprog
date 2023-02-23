# Interprog

[![codecov](https://codecov.io/gh/ThatXliner/interprog/branch/main/graph/badge.svg)](https://codecov.io/gh/ThatXliner/interprog) [![Documentation Status](https://readthedocs.org/projects/interprog/badge/?version=latest)](https://interprog.readthedocs.io/en/latest/?badge=latest) [![CI](https://github.com/ThatXliner/interprog/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ThatXliner/interprog/actions/workflows/ci.yml) [![PyPI](https://img.shields.io/pypi/v/interprog)](https://pypi.org/project/interprog)

> Inter-process progress reports made easy

> **Note**
> If you're already using an RPC framework such as JSON-RPC or gRPC, you probably shouldn't be using this but instead implementing progress reporting within your existing framework. See [this StackOverflow post](https://stackoverflow.com/questions/64352861/is-there-a-way-to-get-progress-messages-from-grpc-request) for ideas.

## Installation

This project was originally written in Python, now with a Rust port. See the corresponding directories in this monorepo for installation instructions.

- [Python](./src/python)
- [Rust](./src/rust)

## License

Copyright © 2022, Bryan Hu

This project is licensed under the [MIT](https://github.com/ThatXliner/interprog/blob/main/LICENSE.txt).
