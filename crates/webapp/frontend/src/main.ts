import { mount } from 'svelte'
import App from './App.svelte'
import { ApiClient } from './lib/api/ApiClient';

import './app.less'

const app = mount(
    App,
    {
        target: document.getElementById('app')!,
    }
);
export default app;

export const api: ApiClient = ApiClient.local();
