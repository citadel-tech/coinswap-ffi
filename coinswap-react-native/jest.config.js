module.exports = {
  testEnvironment: 'node',
  testMatch: ['**/tests/jest/**/*.test.ts'],
  moduleNameMapper: {
    '^react-native$': '<rootDir>/tests/jest/react-native-mock.ts',
  },
  transform: {
    '^.+\\.(ts|tsx)$': [
      'ts-jest',
      {
        tsconfig: 'tsconfig.jest.json',
      },
    ],
  },
  moduleFileExtensions: ['ts', 'tsx', 'js', 'json'],
}
