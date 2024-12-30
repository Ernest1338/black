---
title: 'Instalation'
next: docs/getting-started
weight: 1
---

{{< tabs items="Linux,MacOS,Windows" >}}

  {{< tab >}}
  Clone the `Ernest1338/black` repository:
  ```bash
  $ git clone https://github.com/Ernest1338/black.git
  ```
  Navigate to the project directory:
  ```bash
  $ cd black
  ```
  Use `cargo` to build the project:
  ```bash
  $ cargo build --release
  ```
  Black compiler / interpreter should now be accessible as a build aftifact at `./target/release/black`
  {{< /tab >}}


  {{< tab >}}
  MacOS instructions are the same as linux.

  Clone the `Ernest1338/black` repository:
  ```bash
  $ git clone https://github.com/Ernest1338/black.git
  ```
  Navigate to the project directory:
  ```bash
  $ cd black
  ```
  Use `cargo` to build the project:
  ```bash
  $ cargo build --release
  ```
  Black compiler / interpreter should now be accessible as a build aftifact at `./target/release/black`
  {{< /tab >}}


  {{< tab >}}
  I am not actively using windows. I don't know if black compiles and or runs on this system. \
  Either way instalation steps should be similar to unix, clone the source code, compile the code using cargo and use the resulting binary.

  **YOU ARE ON YOUR OWN**: Good luck.
  {{< /tab >}}

{{< /tabs >}}

## Get a prebuilt binary from Github Actions

TODO

## Install QBE backend

TODO

## Test if the compiler / interpreter is working

While beeing in the `target/release` directory (or any other containing the black binary) execute:
```bash
$ ./black -V
```

You should see version information printed to the screen, similar to this:
```
Black version: v0.0.1
```

Now you can continue learning basics of the black lang, go to the [Getting Started](/docs/getting-started) page for next steps.
