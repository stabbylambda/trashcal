import * as dotenv from "dotenv";
dotenv.config();
export function getEnv(name: string): string {
  if (!process.env[name]) {
    console.log(`${name} environment variable is not set`);
  }

  return process.env[name] ?? process.exit(1);
}
