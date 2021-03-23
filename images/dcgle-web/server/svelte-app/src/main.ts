import App from './App.svelte';

const app = new App({
	target: document.body,
	props: {
    graphqlUrl: process.env.GRAPHQL_SERVER_URL || '/graphql',
	}
});

export default app;
