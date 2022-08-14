import {
  createConnectTransport,
  createPromiseClient,
} from "@bufbuild/connect-web";
import React, { useState } from 'react';
import { People } from './components/people';

// Import service definition that you want to connect to.
import { GreetService } from "../gen/greet/v1/greet_connectweb";

// holy crap I'm bad at front end
function App() {
  const [names, setNames] = useState<string[]>([]);
  
  const getNames = async () => {
    const transport = createConnectTransport({
      baseUrl: "http://localhost:8080",
    });

    const client = createPromiseClient(GreetService, transport);
    for await (const res of client.greetStream({})) {
      setNames(names.concat(names,res.people));
    }
  };

  getNames();

  let nameList: JSX.Element[]=[];
  names.forEach((item,index)=>{
    nameList.push( <People key={index} name={item} />);
  })

  return <>
    {nameList}
  </>;
}

export default App
