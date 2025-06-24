---
type : post
title: Vibe learning rust by building an app
summary: Needed a simple tool for cleaning venvs so decided to write one in rust using vibe coding
linktitle : VenvCleaner
date : "2025-04-17"
tags: [Rust,Zed, Teaching,PySide6,LLM,Deployment,AI]
toc : true
---

# Introduction

I have decided to write a simple .venv cleaning tool to use in the labs. I decided to use rust (as evey new python tool is written in rust as it's blazingly fast (tm) ). I don't know rust, but I'm hoping Claude sonnet 4 does. I will be vibe coding this app using the techniques I discovered when writing my [Apps'ere](https://nccastaff.bournemouth.ac.uk/jmacey/post/AppsEre/AppsEre/) tool.

## Setup

I already rust installed on my mac using [rustup](https://rustup.rs/) and I will again be using the zed editor.  In the last demo I used copilot as my AI, this time I am going to sign up for the free trial of Claude Sonnet 4 that comes with Zed to give this a try.


# .rules file

To start with I have been doing some [RTFM](https://xkcd.com/293/) and discovered the zed [rules](https://zed.dev/docs/ai/rules) file which allows us to communicate set criterial with the agent. I have written the initial basic .rules file

{{< highlight text >}}

You are to use rust as the programming language.
You can use any libraries that are required and these  can be installed via cargo

All code should have full comments and explainations and use the rust styleguide

This app is designed to work in three modes

1. cli with command lines
2. tui with simple interface
3. gui which will use Qt6 rust bindings

{{< /highlight >}}


# Design Document

Most articles on [vibe coding](https://zapier.com/blog/how-to-vibe-code/#prd) recommend writing a Product Requirments Documet [PRD](https://www.productplan.com/glossary/product-requirements-document/) I have written a very simple one to get started.

<details>
<summary>PRD click to expand</summary>

# Venv cleaner

## cli mode

Is an 3 mode application to help manage and clean up .venv folders on mac and linux.

The fist mode is cli, this allows the user to run the tool venv_cleaner [dir] with the following command lines

-r recurse and search from the current directory
-f force always delete

The tool will search for a .venv in the current folder or one passed and prompt the user if found if it should be deleted. If yes then the .venv folder will be removed. If the -r or -f flags are used

-q query

will recurse the directory passed or current directory and print out the path / and size of the .venv folder. The size should be reported in either Mb or Gb depending upon the size.

## tui

This will use the rust tui library to give a console based application. this is activated using the --tui flag.

It will open the current folder and present a view of each .venv folders found by recursing.

It will display in a list the following information

location  size in bytes last used data created date

There should be the ability to open a new folder , delete the selected .venv in the list as well as order by date of creation

## Gui

This will as the same functionality as the tui version but use a Qt6 based GUI rather than a tui.

</details>


## Getting Started

With all this in place I sent the agent a message, including the .rules and Design.md

{{< highlight text >}}
Write the initial project files and folders for the venv cleaner app. Start with the cli mode described.
{{< /highlight  >}}

As I had zed in ["write"](https://zed.dev/docs/ai/agent-panel#built-in-profiles) mode  it went off and wrote a file of stuff for me.

First a Cargo.toml file with all the elements required. Then a src folder with a number of rust files. This took a while so I got a coffee!

It generate 7 files including README.md and a License file. It then asked if it could compile and run the tool to check it. It had writtent a load of unit tests as well.

Compilation actually failed so it went back and fixed it. Finally it compiles with a few warning.

It then asked permission to run the tests. Which worked after the fixes.

## Slow down!

The agent then went a bit mental writing all sorts of scipts to build  the tools and also to install and setup rust none of which were required. It has only written the basic comand line tool which seems to work well.

In the overall build setup there are executables for each of the TUI / GUI mode but none of the TUI / GUI mode stuff actually works.

This initial setup has created a 45k/200k context and used up 28/50 of my prompt usage (I was on about 12 when I started so not too bad).

I've not actually looked at any of the code generated or how the project actually works yet!

## TUI mode

I prompted the agent to develop the tui mode for me an let it work whilst I got breakfast (it is 6.30 am). This took about 20 minutes so managed to do the washing up too!

At some stage the Agent asked me to turn on [Burn Mode](https://zed.dev/docs/ai/models#burn-mode) this seems to eat up the free usage I have (44/50) I switched to the pro trial for 14 days to give it a go. 
