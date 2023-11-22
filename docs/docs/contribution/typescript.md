## Conditional class names
When adding class names to a react element based on conditions
use [template literals](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals)
in combination with the [`?` operator](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_operator)

### Do:
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

### Don't Do:
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

### Rational
1. Flexible: Can be scaled to any number of conditions
2. First-class javascript features: This approach uses basic language features without any library api to learn
3. JetBrains IDE support: The IDE understands that the template string will produce class names
    and allows quick-navigation to the css class declarations.