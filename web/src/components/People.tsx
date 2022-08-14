import React from 'react'; // we need this to make JSX compile

type PeopleProps = {
  name: string,
}

export const People = ({ name }: PeopleProps) => <aside>
  <h2>{ name }</h2>
</aside>