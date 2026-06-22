import test from 'ava'

import { Taker } from '../index'

test('Taker class is exported from native binding', (t) => {
  t.is(typeof Taker, 'function')
})
