## Linter

[Eslint](https://eslint.org/) (invoked as `yarn lint`) is used as external linter to catch common pitfalls and
anti-patterns.

It is also configured to enforce *some* of our style choices.

Please run `yarn lint` before opening a PR or configure you're IDE to run `eslint` (with our config!) on the fly.

The config is located in `eslint.config.js` using `eslint`'s new config format.

## Formatter

[Prettier](https://prettier.io/) (invoked as `yarn format`) is used as formatter.

Please run `yarn format` before opening a PR or configure you're IDE to run `prettier` (with our config!) on the fly.

The config is located directly in the `package.json`.

## Do's and Don'ts

### Components

Only use Functional components in React for new components.

Rationale: state manipulation and caching can be written much simpler as well as it being what the react developers recommend.

### Conditional class names

When adding class names to a react element based on conditions
use [template literals](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals)
in combination with the [`?` operator](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_operator)

!!! success "Do"

    ```jsx
    {/* The following will produce a <div /> with the class "always".                        */}
    {/* If the `condition1` is `true` the div will also have the class "conditional-class1". */}
    {/* Likewise for `condition2` and "conditional-class2"                                   */}
    <div
        className={`always ${
            condition1 ? "conditional-class1" : ""
        } ${
            condition2 ? "conditional-class2" : ""
        }`}
    />
    ```
    (The concrete indentation should be handled by our auto-formatter `prettier`.)

!!! failure "Don't"

    ```jsx
    <div
        className={"always" + condition1 ? " conditional-class1" : "" + condition2 ? " conditional-class2" : ""}
    />
    ```
    ```jsx
    <div
        className={condition1 ? "always conditional-class1" : "always"}
    />
    ```
    ```jsx
    <div
        className={[
            "always",
            ...(condition1 ? "conditional-class1" : []),
            ...(condition1 ? "conditional-class1" : [])
        ].join(" ")}
    />
    ```

#### Rational

1. Flexible: Can be scaled to any number of conditions
2. First-class javascript features: This approach uses basic language features without any library api to learn
3. JetBrains IDE support: The IDE understands that the template string will produce class names
    and allows quick-navigation to the css class declarations.
