import init, { main as clientMain } from './built/client';

const main = async () => {
    await init();

    clientMain();
};

main();
