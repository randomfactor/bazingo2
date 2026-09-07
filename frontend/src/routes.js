import { wrap } from 'svelte-spa-router/wrap'
import { authState } from './lib/auth.svelte.js'
import Home from './views/Home.svelte'
import Login from './views/Login.svelte'
import About from './views/About.svelte'
import Profile from './views/Profile.svelte'
import NotFound from './views/NotFound.svelte'

export const routes = {
  '/': Home,
  '/login': Login,
  '/about': About,
  '/profile': wrap({
    component: Profile,
    conditions: [() => authState.isAuthenticated],
  }),
  '*': NotFound,
}
