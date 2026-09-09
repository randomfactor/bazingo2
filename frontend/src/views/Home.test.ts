import { render, screen } from '@testing-library/svelte'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import Home from './Home.svelte'

describe('Home', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }))
  })

  it('renders the welcome message for signed-out users', () => {
    render(Home)

    expect(screen.getByRole('heading', { name: 'Rust / Svelte Skeleton Application' })).toBeTruthy()
    expect(screen.getByRole('link', { name: 'Sign In' }).getAttribute('href')).toBe('#/login')
  })
})