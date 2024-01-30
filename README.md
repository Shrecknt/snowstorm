# Snowstorm

mass surveillance for block game

![banner](assets/banner.jpg)

### Yet another Minecraft Server Scanner

Scan the entire ipv4 address space for Minecraft servers, with dynamic scanning to find servers on non-default ports.

> **Important**
> - The scanner is currently very early in development and is missing many features
> - The current version of the scanner is synchronous and single threaded, making it extremely slows

## Prerequisites

- Rust 1.77.0-nightly
- Node.js 21.1.0 - for webui
- npm 10.2.4 - for webui
- A postgres database

Other versions of some of the listed programs will probably work but it is not recommended. Rust must be nightly as this program uses async traits which are not in the current latest stable release.

## Installation

Create a postgres database using the [postgres setup script](postgres_setup.sql)

I was too lazy to figure out how environment variables work with svelte, so you are going to have to manually change some stuff in the webui. Try searching for all instances of 'shrecked.dev' in the
`web/` directory and replace that with whatever works best for you.

```sh
git clone https://git.shrecked.dev/Shrecknt/snowstorm.git
cd snowstorm
cp .env.example .env
nano .env # modify .env to your liking
cd web
npm run build
cd ..
cargo r -r --bin snowstorm
```