import { dirname, fromFileUrl, resolve } from "@std/path";

export const namesOfShippedPlugins = async (): Promise<string[]> => {
  const functionsDir = dirname(fromFileUrl(import.meta.url));
  const files = Deno.readDir(functionsDir);
  const plugins: string[] = [];
  for await (const file of files) {
    if (file.isFile && file.name.endsWith(".ts") && file.name !== "mod.ts") {
      plugins.push(file.name.replace(".ts", ""));
    }
  }
  return plugins;
};

export const getPluginFilePath = (functionName: string) => {
  const functionsDir = dirname(fromFileUrl(import.meta.url));
  return resolve(functionsDir, functionName + ".ts");
};
