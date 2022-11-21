/* @jsx h */
import init, { main as rendererMain } from './built/renderer';

import { FunctionComponent, render, h } from 'preact';
import { useEffect, useMemo, useRef, useState } from 'preact/hooks';

interface Body {
    x: number,
    y: number,
    z: number,
    sx: number,
    sy: number,
    sz: number
}

interface Entity {
    id: number,
    body: Body | null
}

interface Dispatcher {
    createEntity(position: { x: number, y: number });
}

const useBinding = (): [Entity[], Dispatcher] => {
    const [entities, setEntities] = useState<Entity[]>([]);

    const chan = useMemo(() => new BroadcastChannel('woods'), []);

    const dispatcher = useMemo((): Dispatcher => {
        const tx = (data: unknown) => chan.postMessage(JSON.stringify(data));

        return {
            createEntity: position => tx({ CreateEntity: {
                ...position,
                z: 0,
                sx: 10,
                sy: 10,
                sz: 10
            } })
        };
    }, []);

    useEffect(() => {
        const handleState = (event: MessageEvent) => {
            return;
            const payload = JSON.parse(event.data);

            const parseComponent = (entity: Entity, comp: object) => {
                const key = Object.keys(comp)[0];
                const data = comp[key];

                switch (key) {
                    case 'Body': {
                        entity.body = data;
                        break;
                    }
                }
            }

            const entities: Entity[] = [];
            for (const item of payload) {
                let entity: Entity = {
                    id: item[0],
                    body: null
                };

                for (const comp of item[1]) {
                    parseComponent(entity, comp);
                }

                entities.push(entity);
            }

            setEntities(entities);
        };

        chan.addEventListener('message', handleState);

        return () => {
            chan.removeEventListener('message', handleState);
            chan.close();
        };
    }, []);

    return [entities, dispatcher];
};

const project = ({ x, y, z }) => {
    return { x, y, z };
};

const View: FunctionComponent<{}> = () => {
    const [entities, dispatcher] = useBinding();

    useEffect(() => {
        const mouseProject = (event: MouseEvent): { x: number, y: number } => {
            return { x: event.pageX, y: event.pageY };
        };

        const handleClick = (event: MouseEvent) => {
            dispatcher.createEntity(mouseProject(event));
        };

        window.addEventListener('click', handleClick);

        return () => {
            window.removeEventListener('click', handleClick);
        }
    });

    return (
        <svg>
            { entities.map((entity) => (
                entity.body &&
                <rect
                    key={ entity.id }
                    x={ project(entity.body).x }
                    y={ project(entity.body).y }
                    width={ 10 }
                    height={ 10 }
                />
            )) }
        </svg>
    )
};

const main = async () => {
    new Worker('/static/worker.js');

    init().then(() => rendererMain());

    const mount = document.querySelector('#mount');
    if (!mount) throw new Error('no mount');

    render(<View/>, mount);
};

main();
