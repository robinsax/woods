import { Application, Router } from 'https://deno.land/x/oak@v6.0.1/mod.ts';

const info = (...args: unknown[]) => {
    console.log('%c[front]', 'background-color: aqua', ...args);
};

const envConfig = () => {
    const envPort = Deno.env.get('FRONT_PORT');
    if (typeof envPort != 'string' || isNaN(+envPort)) {
        info('error: invalid port');
        Deno.exit(1);
    }

    return {
        port: +envPort
    };
};

const { port } = envConfig();

export const main = async () => {
    const app = new Application();

    const router = new Router();

    const index = await Deno.readFile('./index.html');
    router.get('/', ctx => {
        ctx.response.body = index;
    });

    const statics: Record<string, Uint8Array> = {};
    router.get('/static/:fn', async (ctx, next) => {
        const fn = ctx.params.fn || '';
        const isJs = fn.endsWith('js');

        if (!(fn in statics)) {
            info('try load', fn);
            try {
                // TODO: Insecure.
                statics[fn] = await Deno.readFile(`./built/${ fn }`);
            }
            catch {
                next();
                return;
            }

            if (isJs) {
                const encoder = new TextEncoder();
                const decoder = new TextDecoder();
                const updatedSource = decoder.decode(statics[fn])
                    .replace('client_bg.wasm', 'http://localhost/static/client_bg.wasm')
                    .replace('renderer_bg.wasm', 'http://localhost/static/renderer_bg.wasm');

                statics[fn] = new Uint8Array(encoder.encode(updatedSource));
            }
        }
        ctx.response.body = statics[fn];
        ctx.response.headers.set('Content-Type', `application/${ isJs ? 'javascript' : 'wasm' }`);
    });

    app.use(router.routes());

    info(`booting front server on ${ port }`);
    await app.listen({ port });
};

await main();
