type Req = {
    count: number;
};
type Next = () => void;
type Handler = (req: Req, next: Next) => void;

function handle(req: Req, handlers: Handler[]) {
    handlers.slice(1);
    const combined = handlers.slice(1).reduce((acc, curr) => {
        return (req, next) => acc(req, () => curr(req, next));
    }, handlers[0]);
    return combined(req, () => null);
}

function handler(req: Req, next: Next) {
    req.count++;
    next();
}

let handlers = Array(10).fill(handler);

const request = { count: 0 };
handle(request, handlers);

console.log(request);
