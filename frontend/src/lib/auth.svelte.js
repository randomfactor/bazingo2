export const authState = $state({
  isAuthenticated: false,
  isLoading: true,
  user: null,
})

export async function checkSession() {
  authState.isLoading = true

  try {
    const response = await fetch('/api/me', {
      method: 'GET',
      credentials: 'include',
    })

    if (response.ok) {
      const data = await response.json()
      authState.user = data
      authState.isAuthenticated = true
      return
    }

    authState.user = null
    authState.isAuthenticated = false
  } catch (error) {
    console.warn('Session check failed:', error)
    authState.user = null
    authState.isAuthenticated = false
  } finally {
    authState.isLoading = false
  }
}

export async function logout() {
  try {
    await fetch('/auth/logout', {
      method: 'POST',
      credentials: 'include',
    })
  } catch (error) {
    console.warn('Logout request failed:', error)
  } finally {
    authState.user = null
    authState.isAuthenticated = false
  }
}
