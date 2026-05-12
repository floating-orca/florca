import type { PluginRequestBody, ResponseBody } from "@florca/fn";
import { encodePNG } from "jsr:@img/png@0.1.6";

const rawData = await new Response(ReadableStream.from(async function* () {
  for (let r = 0; r < 256; ++r) {
    for (let c = 0; c < 256; ++c) {
      yield Uint8Array.from([255 - r, c, r, 255]);
    }
  }
}())).bytes() as Uint8Array<ArrayBuffer>;

export default async (
  _requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  const png = await encodePNG(rawData, {
    width: 256,
    height: 256,
    compression: 0,
    filter: 0,
    interlace: 0,
  });
  const base64 = btoa(String.fromCharCode(...png));
  return {
    payload: base64,
  };
};
