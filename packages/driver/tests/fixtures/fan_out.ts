export default async ({ context }: any) => ({
  payload: await Promise.all([
    context.run("failing", 1),
    context.run("hanging", 2),
  ]),
});
