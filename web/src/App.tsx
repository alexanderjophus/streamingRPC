import {
  createConnectTransport,
  createPromiseClient,
  ConnectError,
} from "@bufbuild/connect-web";


// Import service definition that you want to connect to.
import { GreetService } from "../gen/greet/v1/greet_connectweb";

// The transport defines what type of endpoint we're hitting.
// In our example we'll be communicating with a Connect endpoint.
const transport = createConnectTransport({
  baseUrl: "http://localhost:8080",
});

// Here we make the client itself, combining the service
// definition with the transport.
const client = createPromiseClient(GreetService, transport);

for await (const res of client.greetStream({})) {
  console.log(res);
}

function App() {
  return <>Hello world</>;
}

export default App
