import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'
import type { Plugin } from 'vite'

function mockApiPlugin(): Plugin {
  return {
    name: 'vite-plugin-mock-api',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = req.url ?? '/'

        if (req.method === 'GET' && url === '/api/users') {
          res.setHeader('Content-Type', 'application/json')
          res.statusCode = 200
          res.end(
            JSON.stringify([
              { id: 1, name: 'Alice', role: 'Engineer' },
              { id: 2, name: 'Bob', role: 'Designer' },
            ]),
          )
          return
        }

        if (req.method === 'GET' && url === '/api/games/me') {
          let body = `        {
          "id": "9f2c1d8c-3d5d-4a90-b69a-460df20d3cb4",
          "name": "Game 9f2c1d8c",
          "number_of_turns": 31,
          "current_turn": 0,
          "current_piece": [
            [false, true, false],
            [true, true, false],
            [false, false, false]
          ],
          "gameboard": [
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false]
          ],
          "player_turn": 0,
          "time_remaining_seconds": 60
        }`

          req.on('data', (chunk: string | Buffer) => {
            body += chunk.toString()
          })

          req.on('end', () => {
            const data = JSON.parse(body || '{}') as Record<string, unknown>
            res.setHeader('Content-Type', 'application/json')
            res.statusCode = 201
            res.end(JSON.stringify({ status: 'created', item: data }))
          })
          return
        }

        if (req.method === 'GET' && url === '/api/me') {
          res.setHeader('Content-Type', 'application/json')
          res.statusCode = 200
          res.end(
            JSON.stringify({
              id: 'dummy-user-id',
              email: 'dummy@example.com',
              name: 'dummy',
            }),
          )
          return
        }

        next()
      })
    },
  }
}

export default defineConfig({
  plugins: [svelte(), mockApiPlugin()],
})
