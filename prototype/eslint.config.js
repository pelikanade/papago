import tseslint from 'typescript-eslint'
import eslintReact from '@eslint-react/eslint-plugin'
import globals from 'globals'

// Prototype lint: typescript-eslint strict + @eslint-react (Rel1cx).
// Local rule: React effects are banned — side effects are driven by the zustand
// state machine or the module bootstrap (main.tsx), never by useEffect.
export default tseslint.config(
  { ignores: ['dist/**', 'node_modules/**'] },
  ...tseslint.configs.strict,
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: { globals: globals.browser },
    plugins: eslintReact.configs['recommended-typescript'].plugins,
    rules: {
      ...eslintReact.configs['recommended-typescript'].rules,
      // Timeline/mock rows carry no ids and are append-only, so index keys are stable here.
      '@eslint-react/no-array-index-key': 'off',
      'no-restricted-syntax': [
        'error',
        {
          selector: "ImportSpecifier[imported.name='useEffect']",
          message: 'useEffect is banned in the prototype — drive side effects from the store or main.tsx.',
        },
        {
          selector: "CallExpression[callee.name='useEffect']",
          message: 'useEffect is banned in the prototype — drive side effects from the store or main.tsx.',
        },
      ],
    },
  },
)