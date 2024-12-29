---
title: 'Getting Started'
prev: docs/instalation
weight: 2
---

Quick tutorial on the black language. Use it to quickly get up to speed writing black code.

{{< callout type="info" >}}
  NOTE: Make sure you have `black` installed before you continue \
  If not, follow the instalation instructions available at: [Instalation](/docs/instalation)
{{< /callout >}}

## Introduction

Let's start with a simple "Hello, World!"

```javascript {filename="main.blk"}
print("Hello, World!")
```

Run it using the following command:
```bash
$ black -i main.blk
```

This will use the interpreter to execute the source code file without a compilation step. \
You should see the familiar "Hello, World!" printed to the screen.

```
Hello, World!
```

Let's now take a look at a longer example.

```javascript {filename="main.blk"}
// print function call
print("hello, world")

// printing multiple arguments
print("hello", 123)

// variable declaration
let greeting = "hello, sailor"

// print binary expression
print(1 + 2 + 3)

// printing variable
print(greeting)

// creating a variable containing bin expr and printing it
let sum = 1 + 1
print(sum)

// adding two variables together
let a = 1
let b = 2
let c = a + b // inline comment
print(c, a + b)
```
