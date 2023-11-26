# Snowstorm

mass surveillance for block game

![banner](assets/banner.png)

### Yet another Minecraft server scanner

Scan the entire ipv4 address space for Minecraft servers, with dynamic scanning to find servers on non-default ports.

> **Important**
> The scanner is currently very early in development and is missing many features.
> The current version of the scanner does not ping real servers, it only pulls data from csv files.
> The scanner also does not save any results to the database.

## Prerequisites

Snowstorm requires the latest stable version of rust, as well as a postgres database. The postgres database can be run on a different server from the scanner.

## Installation

Create a postgres database using the [postgres setup script](postgres_setup.sql)

```sh
git clone https://git.shrecked.dev/Shrecknt/snowstorm.git
cd snowstorm
cp .env.example .env
nano .env # modify .env to match your postgres setup
cargo run --release
```