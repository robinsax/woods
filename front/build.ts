import { build } from 'https://deno.land/x/esbuild@v0.15.14/mod.js';

const main = async () => {
    try {
        await Promise.all([
            build({
                entryPoints: ['window.tsx'],
                bundle: true,
                outfile: 'built/window.js',
            }),
            build({
                entryPoints: ['worker.ts'],
                bundle: true,
                outfile: 'built/worker.js',
            })
        ]);
    }
    catch (err) {
        console.error(err);
        Deno.exit(1);
    }

    Deno.exit(0);
};

await main();
