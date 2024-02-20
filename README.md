# Snowstorm

mass surveillance for block game

![banner](assets/banner.jpg)

### Yet another Minecraft Server Scanner

Scan the entire ipv4 address space for Minecraft servers, with dynamic scanning to find servers on non-default ports.

> **Important**
> - The scanner is currently very early in development and is missing many features
> - The scanner's adaptive scanning capabilities rely on already having some data to expand upon. If you do not already have a small number of servers in your database, the scanner will likely crash.

## Prerequisites

- Rust 1.78.0-nightly
- Node.js 21.1.0 - for webui
- npm 10.2.4 - for webui
- A postgres database

Other versions of some of the listed programs will probably work but it is not recommended.

## Installation

Create a postgres database using the [postgres setup script](postgres_setup.sql)

I was too lazy to figure out how environment variables work with svelte, so you are going to have to manually change some stuff in the webui. Try searching for all instances of 'shrecked.dev' in the
`web/` directory and replace that with whatever works best for you.

```sh
git clone https://git.shrecked.dev/Shrecknt/snowstorm.git
cd snowstorm
cp Snowstorm.toml.example Snowstorm.toml
nano Snowstorm.toml # modify Snowstorm.toml to your liking
cd web
npm run build
cd ..
cargo r -r --bin snowstorm
```