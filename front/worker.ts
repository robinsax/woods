import init, { main as clientMain } from './built';

const main = async () => {
    await init();

    clientMain();
};

main();
