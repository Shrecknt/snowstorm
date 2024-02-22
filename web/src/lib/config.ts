import fs from "node:fs";
import toml from "toml";

const configFile = fs.readFileSync("../Snowstorm.toml");

export const config: Config = toml.parse(configFile.toString());
export type Config = {
    database_url: string,
    web: {
        enabled: boolean,
        listen_uri: boolean,
        domain: string,
        oauth: {
            discord: {
                enabled: boolean,
                client_id: string
            },
            forgejo: {
                enabled: boolean,
                client_id: string
            }
        }
    }
};