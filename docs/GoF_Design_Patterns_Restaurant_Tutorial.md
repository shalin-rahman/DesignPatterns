# GoF Dessign Patterns — Story of a Restaurant

<img width="1024" height="559" alt="image" src="https://github.com/user-attachments/assets/eaea9d08-135b-48c1-a593-9a3033dfbdb5" />


# Part 1: Creational — 5

1. Singleton
2. Factory
3. Abstract Factory
4. Builder
5. Prototype

<img width="1024" height="559" alt="image" src="https://github.com/user-attachments/assets/5b20ac11-eee5-4155-961d-2b613b264741" />


Let's build the understanding from the ground up.

Imagine you're opening a restaurant.

At first, everything is simple:

```text
new Pizza()
new Chef()
new Oven()
```

But as the restaurant grows, creation gets complicated:

```text
Which chef?
Which cuisine?
Which oven?
Which payment system?
Which configuration?
Which objects must be shared?
Which objects are expensive to create?
Which objects have optional settings?
Do two objects need to be compatible?
Do I need a copy of an existing object?
```

That's where **creational design patterns** become useful.

They don't mean:

> "Never use `new`."

They mean:

> **"Let's control object creation when uncontrolled creation is making the design difficult to change, test, or maintain."**

---

# The five patterns in one restaurant

Keep this mental picture throughout the tutorial:

```text
Singleton
"I need ONE kitchen manager."

Factory Method
"The restaurant type decides WHICH chef to create."

Abstract Factory
"Give me the COMPLETE Italian or Bengali kitchen family."

Builder
"Build my CUSTOM pizza step by step."

Prototype
"Give me ANOTHER pizza just like this existing one."
```

The five patterns solve **different creation problems**.

---

# 1. Singleton

**At a glance**

Singleton makes sure a class has only one object and gives the whole program one shared way to reach it.

**Key components**

- **Singleton class**: `KitchenManager`: holds the single instance and hides its constructor.
- **Static access method**: `KitchenManager.getInstance()`: creates the object the first time, then returns the same one.
- **Client**: `RestaurantService`: asks for the manager instead of calling `new`.

**Examples**

- **Restaurant**: A restaurant has one kitchen manager. Every waiter who asks "who runs the kitchen?" gets the same person.
- **Common software**: A logger or a configuration object that the whole app reads from one place.

**Pros**

- Only one instance exists, so shared state stays consistent.
- The object is created only when first needed (lazy loading).
- Gives one global access point.

**Cons**

- It is global state in disguise, so code becomes tightly coupled to it.
- Hard to replace with a fake in unit tests.
- Needs care with threads (see the Holder version).

---


## Restaurant story

You open the restaurant.

You decide:

> **"I need one kitchen manager."**

Why?

Imagine accidentally creating:

```text
KitchenManager #1
KitchenManager #2
KitchenManager #3
```

Each manager thinks they control the kitchen.

Manager #1 says:

> "Kitchen closes at 10."

Manager #2 says:

> "Kitchen closes at 11."

Now you have inconsistent state.

The problem isn't merely that there are multiple Java objects.

The problem is that **the application expects one shared authority/state holder**.

---

## Without Singleton

```java
KitchenManager manager1 = new KitchenManager();
KitchenManager manager2 = new KitchenManager();

System.out.println(manager1 == manager2);
```

Output:

```text
false
```

These are two different objects.

---

# Singleton idea

We want:

```text
                  getInstance()
                       │
             ┌─────────▼─────────┐
             │ KitchenManager     │
             │                   │
             │ one instance      │
             └───────────────────┘
                ▲            ▲
                │            │
             caller A     caller B
```

---

## Basic implementation

```java
public class KitchenManager {

    private static KitchenManager instance;

    private KitchenManager() {
    }

    public static KitchenManager getInstance() {

        if (instance == null) {
            instance = new KitchenManager();
        }

        return instance;
    }
}
```

Use it:

```java
public class Main {

    public static void main(String[] args) {

        KitchenManager a = KitchenManager.getInstance();
        KitchenManager b = KitchenManager.getInstance();

        System.out.println(a == b);
    }
}
```

Output:

```text
true
```

---

## What's actually happening?

### 1. `private static KitchenManager instance`

```java
private static KitchenManager instance;
```

The class has one static variable capable of holding the instance.

`static` means it belongs to the class rather than each individual object.

---

### 2. Private constructor

```java
private KitchenManager() {
}
```

This is extremely important.

Normally:

```java
new KitchenManager();
```

would be possible.

But because the constructor is private:

```java
new KitchenManager();
```

cannot be called from outside the class.

So the class controls its own creation.

---

### 3. `getInstance()`

```java
public static KitchenManager getInstance()
```

This becomes the controlled entrance to the object.

First call:

```text
instance == null
       ↓
create object
       ↓
store object
       ↓
return object
```

Later call:

```text
instance != null
       ↓
return existing object
```

---

# But there's a concurrency problem

Suppose two threads arrive simultaneously:

```text
Thread A                  Thread B

instance == null          instance == null
       │                         │
       ▼                         ▼
new Manager()             new Manager()
```

You could accidentally create two objects.

So the basic implementation isn't sufficient for multithreaded applications.

A modern Java approach is:

```java
public class KitchenManager {

    private KitchenManager() {
    }

    private static class Holder {
        private static final KitchenManager INSTANCE =
                new KitchenManager();
    }

    public static KitchenManager getInstance() {
        return Holder.INSTANCE;
    }
}
```

This uses Java class initialization guarantees to safely initialize the instance.

---

## But should you actually use Singleton?

Be careful.

Singleton creates **global state**.

For example:

```java
KitchenManager.getInstance()
```

can be called from anywhere.

That creates a hidden dependency.

Testing also becomes harder because tests may share the same state.

That's why modern applications often prefer **dependency injection**.

For example:

```java
class RestaurantService {

    private final KitchenManager manager;

    RestaurantService(KitchenManager manager) {
        this.manager = manager;
    }
}
```

Now the dependency is obvious.

Spring can also manage an object as a singleton-scoped bean without you implementing the Singleton pattern yourself.

### Remember

> **Singleton = control the application so there is one shared instance.**

Not:

> "Singleton is the best way to share objects."

Those are very different statements.

---

# 2. Factory Method

**At a glance**

Factory Method lets a parent class define the steps of creating an object, while subclasses decide which concrete class to create.

**Key components**

- **Product**: `Chef`: the interface every chef follows.
- **Concrete Products**: `ItalianChef`, `BengaliChef`: the real chefs.
- **Creator**: abstract `Restaurant`: declares `createChef()` and uses the chef it gets back.
- **Concrete Creators**: `ItalianRestaurant`, `BengaliRestaurant`: each overrides `createChef()` to hire its own chef.

**Examples**

- **Restaurant**: The restaurant knows how to serve an order. Each branch decides which chef it hires.
- **Common software**: A document editor where `Application.createDocument()` returns a `TextDocument` or a `SpreadsheetDocument`, depending on the app subclass.

**Pros**

- The client code depends on `Chef`, not on concrete chef classes.
- A new cuisine is a new subclass. Old code stays unchanged (Open/Closed).
- Creation code lives in one place per branch.

**Cons**

- One new creator subclass for every new product type.
- More classes than a plain `new` call for simple cases.

---


Now the restaurant grows.

You offer:

```text
Italian cuisine
Bengali cuisine
```

You need a chef.

But which chef?

```text
ItalianChef
BengaliChef
```

The restaurant type should determine that.

---

# Without Factory Method

You might write:

```java
Chef chef;

if (type.equals("italian")) {
    chef = new ItalianChef();
} else if (type.equals("bengali")) {
    chef = new BengaliChef();
}
```

This works.

But imagine 20 restaurant types.

Now you have:

```java
if (...)
else if (...)
else if (...)
else if (...)
else if (...)
...
```

And potentially the same logic appears in multiple places.

---

# First create the product abstraction

```java
interface Chef {
    void cook();
}
```

Concrete products:

```java
class ItalianChef implements Chef {

    @Override
    public void cook() {
        System.out.println("Italian chef is cooking pasta.");
    }
}
```

```java
class BengaliChef implements Chef {

    @Override
    public void cook() {
        System.out.println("Bengali chef is cooking biryani.");
    }
}
```

Now:

```text
Chef
 ├── ItalianChef
 └── BengaliChef
```

---

# Now the interesting part

Create an abstract restaurant:

```java
abstract class Restaurant {

    public void prepareMeal() {

        System.out.println("Preparing restaurant meal...");

        Chef chef = createChef();

        chef.cook();
    }

    protected abstract Chef createChef();
}
```

Notice:

```java
Chef chef = createChef();
```

The base class doesn't know which concrete Chef it gets.

It just knows:

> "I need a Chef."

---

## Italian restaurant

```java
class ItalianRestaurant extends Restaurant {

    @Override
    protected Chef createChef() {
        return new ItalianChef();
    }
}
```

## Bengali restaurant

```java
class BengaliRestaurant extends Restaurant {

    @Override
    protected Chef createChef() {
        return new BengaliChef();
    }
}
```

Now:

```java
public class Main {

    public static void main(String[] args) {

        Restaurant italian =
                new ItalianRestaurant();

        Restaurant bengali =
                new BengaliRestaurant();

        italian.prepareMeal();

        bengali.prepareMeal();
    }
}
```

Output:

```text
Preparing restaurant meal...
Italian chef is cooking pasta.

Preparing restaurant meal...
Bengali chef is cooking biryani.
```

---

# Why is this called Factory Method?

This method:

```java
protected abstract Chef createChef();
```

is the **Factory Method**.

The base class says:

> "I need a Chef."

The subclass says:

> "I'll decide which Chef."

```text
Restaurant
    │
    └── createChef()
           │
       subclass decides
           │
      ┌────┴─────┐
      ▼          ▼
 ItalianChef  BengaliChef
```

The important point is **inheritance**.

The creator class defines the factory method, and subclasses override it.

---

# Factory Method vs Simple Factory

This distinction matters.

A **Simple Factory** might look like:

```java
class ChefFactory {

    public static Chef create(String type) {

        if (type.equals("italian")) {
            return new ItalianChef();
        }

        if (type.equals("bengali")) {
            return new BengaliChef();
        }

        throw new IllegalArgumentException();
    }
}
```

That's useful, but traditionally it's called **Simple Factory**, not the GoF Factory Method pattern.

Factory Method looks more like:

```text
abstract Creator
      │
      ├── common workflow
      │
      └── createProduct()
              ▲
              │
       overridden by subclass
```

### Remember

> **Factory Method = subclasses decide which product gets created.**

---

# 3. Abstract Factory

**At a glance**

Abstract Factory gives you an interface for creating families of related objects without naming their concrete classes.

**Key components**

- **Abstract Factory**: `RestaurantFactory`: declares `createChef()`, `createOven()`, `createMenu()`.
- **Concrete Factories**: `ItalianRestaurantFactory`, `BengaliRestaurantFactory`: each builds one matching family.
- **Abstract Products**: `Chef`, `Oven`, `Menu`: one interface per kind of product.
- **Concrete Products**: `ItalianChef`, `ItalianOven`, `ItalianMenu` and `BengaliChef`, `BengaliOven`, `BengaliMenu`.
- **Client**: `Restaurant`: gets a factory and uses only the abstract product types.

**Examples**

- **Restaurant**: An Italian branch must never end up with a Bengali oven. Pick the Italian factory once, and every chef, oven and menu it makes will match.
- **Common software**: A cross-platform UI toolkit. `GUIFactory` has `createButton()` and `createCheckbox()`. `WindowsFactory` makes `WindowsButton` and `WindowsCheckbox`; `MacOSFactory` makes `MacOSButton` and `MacOSCheckbox`. The app code never sees a Windows or Mac class.

**Pros**

- Products from one factory always fit together.
- Client code is kept away from concrete classes.
- A new family (for example `ChineseRestaurantFactory`) is added without changing the client (Open/Closed).

**Cons**

- Many interfaces and classes to write.
- Adding a new kind of product (say `Dessert`) means changing the abstract factory and every concrete factory.

---


Now our restaurant becomes more complicated.

A chef alone isn't enough.

We need:

```text
Chef
Oven
Menu
```

And they must match the cuisine.

For Italian:

```text
ItalianChef
ItalianOven
ItalianMenu
```

For Bengali:

```text
BengaliChef
BengaliOven
BengaliMenu
```

This is a **family**.

That's the key idea behind Abstract Factory.

---

# Product interfaces

```java
interface Chef {
    void cook();
}

interface Oven {
    void bake();
}

interface Menu {
    void show();
}
```

---

# Italian family

```java
class ItalianChef implements Chef {

    @Override
    public void cook() {
        System.out.println("Italian chef: cooking pasta.");
    }
}
```

```java
class ItalianOven implements Oven {

    @Override
    public void bake() {
        System.out.println("Italian oven: baking pizza.");
    }
}
```

```java
class ItalianMenu implements Menu {

    @Override
    public void show() {
        System.out.println("Italian menu.");
    }
}
```

---

# Bengali family

```java
class BengaliChef implements Chef {

    @Override
    public void cook() {
        System.out.println("Bengali chef: cooking biryani.");
    }
}
```

```java
class BengaliOven implements Oven {

    @Override
    public void bake() {
        System.out.println("Bengali oven: baking naan.");
    }
}
```

```java
class BengaliMenu implements Menu {

    @Override
    public void show() {
        System.out.println("Bengali menu.");
    }
}
```

---

# Abstract Factory

```java
interface RestaurantFactory {

    Chef createChef();

    Oven createOven();

    Menu createMenu();
}
```

This is the important interface.

It says:

> "Every restaurant family must be able to provide these related products."

---

# Italian factory

```java
class ItalianRestaurantFactory
        implements RestaurantFactory {

    @Override
    public Chef createChef() {
        return new ItalianChef();
    }

    @Override
    public Oven createOven() {
        return new ItalianOven();
    }

    @Override
    public Menu createMenu() {
        return new ItalianMenu();
    }
}
```

Bengali:

```java
class BengaliRestaurantFactory
        implements RestaurantFactory {

    @Override
    public Chef createChef() {
        return new BengaliChef();
    }

    @Override
    public Oven createOven() {
        return new BengaliOven();
    }

    @Override
    public Menu createMenu() {
        return new BengaliMenu();
    }
}
```

---

# Client

Now create the restaurant:

```java
class Restaurant {

    private final RestaurantFactory factory;

    public Restaurant(RestaurantFactory factory) {
        this.factory = factory;
    }

    public void open() {

        Chef chef = factory.createChef();
        Oven oven = factory.createOven();
        Menu menu = factory.createMenu();

        menu.show();
        chef.cook();
        oven.bake();
    }
}
```

Notice what `Restaurant` does **not** know.

It doesn't know:

```text
ItalianChef
BengaliChef
ItalianOven
BengaliOven
ItalianMenu
BengaliMenu
```

It knows only:

```text
Chef
Oven
Menu
RestaurantFactory
```

Run it:

```java
public class Main {

    public static void main(String[] args) {

        RestaurantFactory factory =
                new ItalianRestaurantFactory();

        Restaurant restaurant =
                new Restaurant(factory);

        restaurant.open();
    }
}
```

Output:

```text
Italian menu.
Italian chef: cooking pasta.
Italian oven: baking pizza.
```

Switch family:

```java
RestaurantFactory factory =
        new BengaliRestaurantFactory();
```

The Restaurant code doesn't change.

---

# Why is Abstract Factory different from Factory Method?

This is the distinction I want you to remember.

## Factory Method

You're asking:

> **"Which Chef?"**

```text
Restaurant
     │
 createChef()
     │
 ┌───┴────┐
 ▼        ▼
Italian  Bengali
Chef     Chef
```

**One product type.**

---

## Abstract Factory

You're asking:

> **"Which entire family?"**

```text
             RestaurantFactory
                    │
           ┌────────┴────────┐
           ▼                 ▼
       Italian            Bengali
       Factory             Factory
           │                 │
       ┌───┼───┐         ┌───┼───┐
       ▼   ▼   ▼         ▼   ▼   ▼
      Chef Oven Menu    Chef Oven Menu
```

**Multiple related product types.**

---

# Where Abstract Factory appears in real software

### UI themes

```text
DarkFactory
 ├── DarkButton
 ├── DarkTextBox
 └── DarkCheckbox

LightFactory
 ├── LightButton
 ├── LightTextBox
 └── LightCheckbox
```

You don't want a dark button with a light theme's components accidentally.

---

### Database family

```text
PostgreSqlFactory
 ├── Connection
 ├── Command
 └── Transaction

SqlServerFactory
 ├── Connection
 ├── Command
 └── Transaction
```

---

### Payment provider family

```text
StripeFactory
 ├── Gateway
 ├── FraudChecker
 └── ReceiptGenerator

BkashFactory
 ├── Gateway
 ├── FraudChecker
 └── ReceiptGenerator
```

---

### Cloud provider

```text
AWSFactory
 ├── Storage
 ├── Queue
 └── Compute

AzureFactory
 ├── Storage
 ├── Queue
 └── Compute
```

The value is not merely "different implementations."

It's that the objects form a **coherent family**.

---

# 4. Builder

**At a glance**

Builder creates a complex object step by step, so you set only the parts you need and get a finished object at the end.

**Key components**

- **Product**: `Pizza`: an immutable object with many optional parts.
- **Builder**: `Pizza.Builder`: has one method per option (`size()`, `crust()`, `cheese()`, `olives()`) and a final `build()`.
- **Client**: the order code: chains the builder calls it needs, then calls `build()`.

**Examples**

- **Restaurant**: A customer orders a large pizza with extra cheese and no olives. The order is built one choice at a time, and the pizza is made once at the end.
- **Common software**: `StringBuilder` in Java, or an HTTP request builder: `HttpRequest.newBuilder().uri(...).header(...).build()`.

**Pros**

- No long constructors with many `null` arguments.
- Code reads like the order itself.
- The finished object can be immutable and checked in `build()`.

**Cons**

- One extra class per product.
- Too much for objects with only two or three fields.

---


Back to the restaurant.

A customer wants:

> Large pizza, thin crust, extra cheese, pepperoni, mushrooms, olives, no onions, spicy sauce...

A naive constructor becomes ugly:

```java
Pizza pizza = new Pizza(
    "large",
    "thin",
    true,
    true,
    true,
    false,
    "spicy",
    ...
);
```

What does this mean?

```java
true, true, true, false
```

You have to remember which boolean means what.

This is the **telescoping constructor problem**: too many constructor parameters, especially optional ones.

---

# Builder solution

We want:

```java
Pizza pizza = new Pizza.Builder()
        .size("large")
        .crust("thin")
        .cheese(true)
        .pepperoni(true)
        .mushrooms(true)
        .olives(true)
        .onions(false)
        .sauce("spicy")
        .build();
```

Now the code explains itself.

---

# Full implementation

```java
import java.util.List;

public final class Pizza {

    private final String size;
    private final String crust;
    private final boolean cheese;
    private final boolean pepperoni;
    private final boolean mushrooms;
    private final boolean olives;
    private final boolean onions;
    private final String sauce;

    private Pizza(Builder builder) {

        this.size = builder.size;
        this.crust = builder.crust;
        this.cheese = builder.cheese;
        this.pepperoni = builder.pepperoni;
        this.mushrooms = builder.mushrooms;
        this.olives = builder.olives;
        this.onions = builder.onions;
        this.sauce = builder.sauce;
    }

    public void printDescription() {

        System.out.println("Pizza:");
        System.out.println("Size: " + size);
        System.out.println("Crust: " + crust);
        System.out.println("Cheese: " + cheese);
        System.out.println("Pepperoni: " + pepperoni);
        System.out.println("Mushrooms: " + mushrooms);
        System.out.println("Olives: " + olives);
        System.out.println("Onions: " + onions);
        System.out.println("Sauce: " + sauce);
    }

    public static class Builder {

        private String size;
        private String crust;
        private boolean cheese;
        private boolean pepperoni;
        private boolean mushrooms;
        private boolean olives;
        private boolean onions;
        private String sauce;

        public Builder size(String size) {
            this.size = size;
            return this;
        }

        public Builder crust(String crust) {
            this.crust = crust;
            return this;
        }

        public Builder cheese(boolean cheese) {
            this.cheese = cheese;
            return this;
        }

        public Builder pepperoni(boolean pepperoni) {
            this.pepperoni = pepperoni;
            return this;
        }

        public Builder mushrooms(boolean mushrooms) {
            this.mushrooms = mushrooms;
            return this;
        }

        public Builder olives(boolean olives) {
            this.olives = olives;
            return this;
        }

        public Builder onions(boolean onions) {
            this.onions = onions;
            return this;
        }

        public Builder sauce(String sauce) {
            this.sauce = sauce;
            return this;
        }

        public Pizza build() {

            if (size == null || size.isBlank()) {
                throw new IllegalStateException(
                        "Pizza size is required"
                );
            }

            if (crust == null || crust.isBlank()) {
                throw new IllegalStateException(
                        "Pizza crust is required"
                );
            }

            return new Pizza(this);
        }
    }
}
```

Use:

```java
public class Main {

    public static void main(String[] args) {

        Pizza pizza = new Pizza.Builder()
                .size("large")
                .crust("thin")
                .cheese(true)
                .pepperoni(true)
                .mushrooms(true)
                .olives(true)
                .onions(false)
                .sauce("spicy")
                .build();

        pizza.printDescription();
    }
}
```

---

# What's happening internally?

The Builder is essentially accumulating configuration:

```text
Builder
  │
  ├── size = large
  ├── crust = thin
  ├── cheese = true
  ├── pepperoni = true
  ├── mushrooms = true
  └── ...
           │
           ▼
        build()
           │
           ▼
         Pizza
```

The `Pizza` constructor is private:

```java
private Pizza(Builder builder)
```

So users cannot casually construct an incomplete Pizza.

They go through:

```java
build()
```

where validation can happen.

---

# Why `return this`?

Consider:

```java
public Builder size(String size) {
    this.size = size;
    return this;
}
```

It returns the same Builder.

Therefore:

```java
builder
    .size(...)
    .crust(...)
    .cheese(...)
```

can be chained.

That's called a **fluent API**.

---

# Why is Pizza immutable?

We use:

```java
private final String size;
```

and don't provide setters.

Once built:

```java
Pizza pizza = ...
```

its configuration cannot be changed.

That's often desirable for value-like objects.

---

# When NOT to use Builder

Don't write:

```java
new User.Builder()
    .name("John")
    .age(30)
    .build();
```

for an object that only has:

```java
User(String name, int age)
```

This is probably simpler:

```java
new User("John", 30);
```

Builder is useful when construction has **enough complexity to justify it**.

---

# 5. Prototype

**At a glance**

Prototype creates new objects by copying an existing object instead of building them from scratch.

**Key components**

- **Prototype**: `Cloneable` with `clone()`: the copy operation.
- **Concrete Prototype**: `Pizza`: knows how to copy itself (shallow or deep copy).
- **Client**: the order code: clones a ready pizza and changes a few fields.

**Examples**

- **Restaurant**: The kitchen keeps a ready "Margherita template". For each order it copies the template and only changes the size or toppings.
- **Common software**: Copying a shape in a drawing app (Ctrl+C, Ctrl+V), or cloning a preset game character.

**Pros**

- Fast when building an object is costly.
- No need to know the concrete class to make a copy.
- Preset objects can serve as templates.

**Cons**

- Deep copy is easy to get wrong when objects hold lists or other objects.
- Java's `clone()` has awkward rules (`Cloneable`, checked exception).

---


Now imagine your restaurant has pizza templates.

You have a carefully configured:

```text
Family Pizza
Large
Thin crust
Extra cheese
Pepperoni
Mushroom
Spicy sauce
```

You want 100 similar pizzas.

Why repeatedly reconstruct the configuration?

Instead:

> **"Take this existing pizza and make another one like it."**

That's Prototype.

---

# The problem with shallow copying

Suppose:

```java
class Pizza {

    String name;
    List<String> toppings;
}
```

If you simply copy the object:

```text
original
  │
  └── toppings ──────┐
                     ▼
                  List A

copy
  │
  └── toppings ──────┘
```

Both objects may point to the **same mutable list**.

Then:

```java
copy.toppings.add("Olives");
```

could unexpectedly modify the original's toppings too.

That's a **shallow-copy problem**.

---

# Deep copy

We want:

```text
original
  │
  └── toppings → List A

copy
  │
  └── toppings → List B
```

The lists are separate.

---

# Java Prototype example

```java
import java.util.ArrayList;
import java.util.List;

class Pizza implements Cloneable {

    private String name;
    private List<String> toppings;

    public Pizza(String name, List<String> toppings) {
        this.name = name;
        this.toppings = new ArrayList<>(toppings);
    }

    public void addTopping(String topping) {
        toppings.add(topping);
    }

    public void print() {
        System.out.println(
                name + " -> " + toppings
        );
    }

    public Pizza copy() {

        return new Pizza(
                this.name,
                this.toppings
        );
    }
}
```

Use:

```java
public class Main {

    public static void main(String[] args) {

        Pizza original = new Pizza(
                "Family Pizza",
                List.of(
                        "Cheese",
                        "Pepperoni",
                        "Mushrooms"
                )
        );

        Pizza copy = original.copy();

        copy.addTopping("Olives");

        original.print();
        copy.print();
    }
}
```

Output:

```text
Family Pizza -> [Cheese, Pepperoni, Mushrooms]

Family Pizza -> [Cheese, Pepperoni, Mushrooms, Olives]
```

The original wasn't modified.

---

# Why?

This constructor:

```java
this.toppings = new ArrayList<>(toppings);
```

creates a new list.

So:

```text
original.toppings → List A

copy.toppings     → List B
```

instead of:

```text
original.toppings ─┐
                   ▼
                 List A
                   ▲
                   │
copy.toppings ─────┘
```

---

# Is Java's `clone()` always the answer?

No.

Java's built-in `Cloneable` mechanism has some awkward historical behavior.

In modern Java, a method such as:

```java
public Pizza copy()
```

can often be clearer and safer than exposing `clone()`.

The **Prototype pattern** is the concept:

> Create a new object by copying an existing object.

It doesn't require you to use `Object.clone()`.

---

# When Prototype makes sense

Imagine a report system:

```text
ReportTemplate
 ├── company logo
 ├── headers
 ├── formatting
 ├── sections
 └── configuration
```

Creating and configuring that from scratch every time could be expensive.

Instead:

```java
ReportTemplate copy =
        template.copy();
```

Then customize it.

Good candidates include:

* document templates
* game objects
* preconfigured reports
* complex configuration objects
* expensive-to-create objects

But if:

```java
new Pizza(...)
```

is cheap and simple, cloning it is probably unnecessary complexity.

---

# Now let's put the five side by side

| Pattern              | Problem                                       | Main idea                 | Creates                   |
| -------------------- | --------------------------------------------- | ------------------------- | ------------------------- |
| **Singleton**        | Need controlled single instance               | One shared instance       | One object                |
| **Factory Method**   | Subclasses need different products            | Subclass decides creation | One product               |
| **Abstract Factory** | Need compatible object families               | Factory selects family    | Multiple related products |
| **Builder**          | Object has complicated construction           | Build step-by-step        | One complex object        |
| **Prototype**        | Creating similar objects is expensive/awkward | Copy existing object      | New copy                  |

---

# The most important distinction

Imagine the restaurant owner asks five different questions.

### Question 1

> "How many kitchen managers should exist?"

**Singleton**

```text
ONE
```

---

### Question 2

> "Which chef should this restaurant create?"

**Factory Method**

```text
ItalianChef
OR
BengaliChef
```

---

### Question 3

> "Which complete cuisine family should this restaurant use?"

**Abstract Factory**

```text
Italian:
Chef + Oven + Menu

OR

Bengali:
Chef + Oven + Menu
```

---

### Question 4

> "How should this complicated pizza be constructed?"

**Builder**

```text
size
 ↓
crust
 ↓
cheese
 ↓
toppings
 ↓
sauce
 ↓
build()
```

---

### Question 5

> "I already have a configured pizza. Can I make another one like it?"

**Prototype**

```text
Existing Pizza
      │
    copy()
      │
      ▼
New Pizza
```

---

# The `if` / `switch` question

You raised this earlier, and it is an important point.

Do these patterns eliminate:

```java
if (...)
else if (...)
switch (...)
```

?

**No.**

The decision still has to happen somewhere.

Suppose configuration says:

```text
restaurant.cuisine = ITALIAN
```

Somewhere the application has to turn that into:

```text
ItalianRestaurantFactory
```

You could have:

```java
if (cuisine == Cuisine.ITALIAN) {
    factory = new ItalianRestaurantFactory();
} else {
    factory = new BengaliRestaurantFactory();
}
```

That's still an `if`.

The difference is **where the decision happens**.

You want:

```text
Configuration
     │
     ▼
Factory selection
     │
     ▼
RestaurantFactory
     │
     ▼
Business code
```

rather than:

```text
OrderService
    │
    ├── if Italian
    ├── if Bengali
    ├── create chef
    ├── create oven
    └── create menu
```

The decision doesn't disappear.

**It gets isolated.**

You can also use:

```java
Map<Cuisine, RestaurantFactory>
```

or dependency injection.

---

# How to recognize the patterns in real code

When you encounter code, ask these questions.

### "There should only be one."

Think:

**Singleton**

But ask whether DI would be better.

---

### "Something needs to decide which implementation to create."

Think:

**Factory**

---

### "A subclass determines which product gets created."

Think:

**Factory Method**

---

### "I need several related implementations that must belong together."

Think:

**Abstract Factory**

---

### "This object has 10 optional construction parameters."

Think:

**Builder**

---

### "I already have a configured object and want another similar object."

Think:

**Prototype**

---

# Now the SOLID connection

These patterns make more sense when you see the principles behind them.

---

## SRP — Single Responsibility

A class should have a focused responsibility.

Bad:

```text
Restaurant
 ├── business logic
 ├── create chef
 ├── create oven
 ├── create menu
 ├── configure database
 └── manage global state
```

Better:

```text
Restaurant
     │
     └── business behavior

RestaurantFactory
     │
     └── object creation

Pizza.Builder
     │
     └── pizza construction
```

Singleton deserves caution here.

If your Singleton becomes:

```text
KitchenManager
 ├── configuration
 ├── database
 ├── logging
 ├── payments
 ├── users
 └── everything else
```

you've created a **God Object**, not good design.

---

# OCP — Open/Closed Principle

> Open for extension, closed for modification.

Suppose tomorrow you add:

```text
Japanese restaurant
```

With Abstract Factory:

```text
JapaneseChef
JapaneseOven
JapaneseMenu
JapaneseRestaurantFactory
```

Existing client:

```java
Restaurant restaurant =
        new Restaurant(factory);
```

doesn't need to know the new concrete classes.

That's a strong example of OCP.

---

# LSP — Liskov Substitution

If:

```java
Chef chef;
```

then:

```java
ItalianChef
```

should behave as a valid `Chef`.

Likewise:

```java
BengaliChef
```

should also be valid.

This would be suspicious:

```java
class BengaliChef implements Chef {

    @Override
    public void cook() {
        throw new UnsupportedOperationException();
    }
}
```

If a class claims to be a `Chef`, but fundamentally can't perform the operation promised by `Chef`, the abstraction is probably wrong.

---

# ISP — Interface Segregation

Instead of:

```java
interface RestaurantComponent {

    void cook();

    void bake();

    void showMenu();

    void processPayment();

    void cleanKitchen();

    void manageEmployees();
}
```

prefer focused interfaces:

```java
interface Chef {
    void cook();
}

interface Oven {
    void bake();
}

interface Menu {
    void show();
}
```

Small interfaces are easier to implement and substitute.

---

# DIP — Dependency Inversion

This:

```java
class Restaurant {

    private ItalianChef chef =
            new ItalianChef();
}
```

couples Restaurant directly to ItalianChef.

Better:

```java
class Restaurant {

    private final Chef chef;

    Restaurant(Chef chef) {
        this.chef = chef;
    }
}
```

Now:

```text
Restaurant
     │
     ▼
   Chef
   ▲   ▲
   │   │
Italian Bengali
```

Restaurant depends on the abstraction.

---

# Composition over inheritance

Factory Method commonly uses inheritance:

```text
Restaurant
   ▲
   │
ItalianRestaurant
```

But Abstract Factory uses composition:

```java
Restaurant restaurant =
        new Restaurant(factory);
```

The restaurant **has a factory** rather than becoming a subclass of a factory.

That can be easier to change at runtime.

For example:

```java
Restaurant restaurant =
        new Restaurant(new ItalianRestaurantFactory());
```

or:

```java
Restaurant restaurant =
        new Restaurant(new BengaliRestaurantFactory());
```

Same Restaurant class.

---

# Encapsulate what varies

This is one of the most useful principles behind these patterns.

Ask:

> **What is likely to change?**

In our restaurant:

```text
Cuisine
Chef
Oven
Menu
Pizza configuration
Object copying
```

Then isolate those variations.

```text
Cuisine variation
       ↓
Abstract Factory

Chef creation variation
       ↓
Factory Method

Pizza construction variation
       ↓
Builder

Pizza duplication
       ↓
Prototype
```

---

# DRY

Factories can prevent repeated construction logic.

Builder can centralize validation.

But don't misunderstand DRY.

This:

```java
new User(name, age)
```

doesn't need a factory simply because you wrote it twice.

**Don't create abstractions merely to eliminate a few repeated lines.**

---

# KISS

This is where developers often misuse design patterns.

If you have:

```java
Pizza pizza = new Pizza("Large", "Thin");
```

don't create:

```text
PizzaFactory
PizzaBuilder
PizzaPrototypeRegistry
PizzaAbstractFactory
PizzaCreationStrategy
```

That's ridiculous.

A pattern is justified by a **design problem**, not by the fact that a pattern exists.

---

# YAGNI

"You aren't gonna need it."

If today your restaurant has:

```text
one Chef
one Oven
one Menu
```

don't build:

```text
ItalianFactory
BengaliFactory
JapaneseFactory
ChineseFactory
IndianFactory
```

before you actually need them.

Design for known variation, not imaginary complexity.

---

# Immutability

Builder works especially well with immutable objects.

For example:

```java
public final class Pizza {

    private final String size;
    private final String crust;
}
```

Once:

```java
Pizza pizza = builder.build();
```

the pizza doesn't suddenly change because some other part of the application modified the Builder.

That gives you safer objects.

---

# A practical decision tree

Here's a slightly more accurate version of the decision process:

```text
             I need to create an object
                       │
                       ▼
       Is object creation itself the problem?
                       │
              ┌────────┴────────┐
              │                 │
             NO                YES
              │                 │
        Use constructor         ▼
                       What problem?
                              │
          ┌───────────────────┼──────────────────┐
          │                   │                  │
       One shared         Complex           Existing object
        instance         construction          to copy
          │                   │                  │
          ▼                   ▼                  ▼
      Singleton            Builder           Prototype
                             
                       Need polymorphic
                       product creation?
                              │
                             YES
                              │
                              ▼
                    One product or family?
                       │             │
                    ONE            FAMILY
                       │             │
                       ▼             ▼
                Factory Method   Abstract Factory
```

One correction to a simplistic decision tree is important:

**"There are variants" does not automatically mean Factory Method.**

You should first ask **why the variation exists and where the creation decision belongs**.

---

# One mini-project using all five

Let's put everything together.

Imagine our restaurant management application.

```text
                    Restaurant App
                         │
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
    KitchenManager    Restaurant      Pizza
      Singleton        Factory        Builder
                         │
                    ┌────┴────┐
                    ▼         ▼
                 Italian    Bengali
                 Factory    Factory
                    │         │
                 Chef/Oven   Chef/Oven
                    │
                    ▼
                  Pizza
                    │
                  copy()
                    │
                    ▼
                Prototype
```

---

## Singleton

```java
KitchenManager manager =
        KitchenManager.getInstance();
```

**Why?**

One shared kitchen-management instance.

---

## Factory Method

```java
abstract class Restaurant {

    public void prepareMeal() {
        Chef chef = createChef();
        chef.cook();
    }

    protected abstract Chef createChef();
}
```

Italian restaurant:

```java
class ItalianRestaurant extends Restaurant {

    @Override
    protected Chef createChef() {
        return new ItalianChef();
    }
}
```

**Why?**

The subclass determines the chef.

---

## Abstract Factory

```java
interface RestaurantFactory {

    Chef createChef();

    Oven createOven();

    Menu createMenu();
}
```

Italian factory:

```java
class ItalianRestaurantFactory
        implements RestaurantFactory {

    public Chef createChef() {
        return new ItalianChef();
    }

    public Oven createOven() {
        return new ItalianOven();
    }

    public Menu createMenu() {
        return new ItalianMenu();
    }
}
```

**Why?**

We need an entire compatible cuisine family.

---

## Builder

```java
Pizza pizza =
        new Pizza.Builder()
                .size("large")
                .crust("thin")
                .cheese(true)
                .pepperoni(true)
                .build();
```

**Why?**

Pizza construction has many optional properties.

---

## Prototype

```java
Pizza secondPizza =
        originalPizza.copy();
```

**Why?**

We want another object based on an existing configuration.

---

# Final mental model

Forget the formal definitions for a moment.

Imagine you're standing in your restaurant kitchen.

Someone asks:

> **"Who manages this kitchen?"**

**Singleton**

> One kitchen manager.

---

Someone asks:

> **"Which chef should this restaurant create?"**

**Factory Method**

> The restaurant type decides.

---

Someone asks:

> **"Which complete kitchen setup are we using?"**

**Abstract Factory**

> Give me the whole Italian family or the whole Bengali family.

---

Someone asks:

> **"How do I construct this highly customized pizza?"**

**Builder**

> Configure it step by step and then build it.

---

Someone asks:

> **"I want another pizza exactly like this one, then I'll customize it."**

**Prototype**

> Copy the existing pizza.

---

## The five words to memorize

```text
Singleton       → ONE

Factory Method  → WHICH ONE?

Abstract Factory → WHICH FAMILY?

Builder         → HOW TO BUILD?

Prototype       → COPY
```

And the bigger lesson is even more important:

> **Design patterns are not goals. They're tools for managing complexity.**

If `new Pizza()` is perfectly clear, use `new Pizza()`.

If a constructor has 15 optional parameters, consider **Builder**.

If subclasses need to choose a product, consider **Factory Method**.

If you need a compatible family of products, consider **Abstract Factory**.

If you need to duplicate configured objects, consider **Prototype**.

If you genuinely require one shared instance, consider **Singleton**—but in dependency-injected applications, let the DI container manage that lifecycle when possible.


# Structural Patterns

**Structural — 7**

6. Adapter
7. Facade
8. Decorator
9. Composite
10. Proxy
11. Bridge
12. Flyweight

#  Behavioral — 11**
13. Strategy
14. Observer
15. Command
16. Template Method
17. State
18. Chain of Responsibility
19. Iterator
20. Mediator
21. Memento
22. Visitor
23. Interpreter

For each one, we’ll use the same structure:

> **WHY → PROBLEM → NAIVE CODE → PATTERN → RESTAURANT STORY → JAVA 17+ CODE → EXECUTION → CODE WALKTHROUGH → BENEFITS → TRADE-OFFS → WHEN TO USE → WHEN NOT TO USE → REAL-WORLD EXAMPLE → COMMON MISTAKES → RELATED SOLID PRINCIPLES**

And we'll keep one **continuous restaurant domain** where that makes the concept clearer, then connect it to real software such as APIs, payment systems, databases, notifications, authentication, logging, UI, etc.

### Recommended learning order

I suggest we do **Structural first**, because they teach an extremely important architectural idea:

```text
CREATIONAL
How do I CREATE objects?
        │
        ▼
STRUCTURAL
How do I COMPOSE objects?
        │
        ▼
BEHAVIORAL
How do objects COLLABORATE?
```

So the continuation starts with:

**Adapter → Facade → Decorator → Composite → Proxy → Bridge → Flyweight**

Then:

**Strategy → Observer → Command → Template Method → State → Chain of Responsibility → Iterator → Mediator → Memento → Visitor → Interpreter**

That order also lets later patterns reuse concepts you've already learned.

---

# PART II — STRUCTURAL DESIGN PATTERNS

<img width="1024" height="559" alt="image" src="https://github.com/user-attachments/assets/240bba54-fd35-4f83-8772-ff93e1549d44" />


## 6. Adapter Pattern

**At a glance**

Adapter lets two classes with incompatible interfaces work together by wrapping one of them.

**Key components**

- **Target**: `PaymentService`: the interface the restaurant already uses.
- **Adaptee**: `BkashService`: the outside service with its own method names.
- **Adapter**: `BkashPaymentAdapter`: implements `PaymentService` and calls `BkashService` inside.
- **Client**: the checkout code: talks only to `PaymentService`.

**Examples**

- **Restaurant**: The restaurant's checkout expects `pay(amount)`. bKash offers `makePayment(currency, amount)`. The adapter translates one call into the other.
- **Common software**: A power plug adapter, or wrapping an old XML API so new code can call it through a JSON-style interface.

**Pros**

- Reuses existing classes without changing them.
- Keeps translation code in one place (Single Responsibility).
- New providers can be added with new adapters.

**Cons**

- One more layer of classes.
- Sometimes simpler to change the service itself, if you own it.

---


### Restaurant story

Imagine your restaurant already works with:

```text
PaymentService
    ↓
pay(amount)
```

Your application expects:

```java
paymentService.pay(500);
```

But a new payment provider gives you:

```java
bkash.makePayment("BDT", 500);
```

You cannot modify your entire restaurant application just because one external system has a different interface.

So you create an **Adapter**.

```text
Your Restaurant
      │
      ▼
PaymentService
      │
      ▼
PaymentAdapter
      │
      ▼
Bkash API
```

The adapter translates one interface into another.

---

## What goes wrong without Adapter?

Suppose:

```java
class BkashService {

    public void makePayment(String currency, double amount) {
        System.out.println(
            "Paid " + amount + " " + currency + " using bKash"
        );
    }
}
```

Your restaurant wants:

```java
interface PaymentService {
    void pay(double amount);
}
```

But:

```java
PaymentService payment = new BkashService();
```

doesn't compile.

The interfaces don't match.

You might be tempted to change all your application code:

```java
bkash.makePayment("BDT", 500);
```

Then another provider arrives:

```java
nagad.sendMoney(500);
```

Now your business code becomes:

```java
if (provider.equals("bkash")) {
    ...
}
else if (provider.equals("nagad")) {
    ...
}
else if (provider.equals("stripe")) {
    ...
}
```

That spreads external-provider knowledge everywhere.

---

# The Adapter solution

```java
interface PaymentService {
    void pay(double amount);
}
```

Existing external service:

```java
class BkashService {

    public void makePayment(String currency, double amount) {
        System.out.println(
            "Paid " + amount + " " + currency + " using bKash"
        );
    }
}
```

Adapter:

```java
class BkashPaymentAdapter implements PaymentService {

    private final BkashService bkash;

    public BkashPaymentAdapter(BkashService bkash) {
        this.bkash = bkash;
    }

    @Override
    public void pay(double amount) {
        bkash.makePayment("BDT", amount);
    }
}
```

Client:

```java
public class Main {

    public static void main(String[] args) {

        PaymentService payment =
            new BkashPaymentAdapter(new BkashService());

        payment.pay(500);
    }
}
```

Output:

```text
Paid 500.0 BDT using bKash
```

### What actually happened?

The restaurant knows:

```java
PaymentService
```

It doesn't need to know:

```java
BkashService
```

The adapter knows both:

```text
Restaurant
    ↓
PaymentService
    ↓
BkashPaymentAdapter
    ↓
BkashService
```

### Why is it called Adapter?

Because it **adapts one interface to another interface**.

Think:

```text
European plug
      ↓
   Adapter
      ↓
Bangladeshi socket
```

The appliance doesn't change.

The socket doesn't change.

The adapter makes them compatible.

---

## Real-world uses

Adapter is extremely common.

### Legacy code

```text
OldCustomerService
       ↓
CustomerServiceAdapter
       ↓
New application
```

### Third-party APIs

```text
Your interface
      ↓
Stripe Adapter
      ↓
Stripe SDK
```

### Database libraries

```text
Application
    ↓
Repository interface
    ↓
PostgreSQL adapter
```

### Java examples

Many Java APIs effectively use adapter-like ideas when wrapping incompatible APIs.

---

## Adapter vs Factory

This distinction is important.

**Factory:**

> Which object should I create?

**Adapter:**

> How do I make this existing object compatible with my interface?

```text
Factory
    ↓
CREATE

Adapter
    ↓
CONVERT/TRANSLATE INTERFACE
```

---

# 7. Facade Pattern

**At a glance**

Facade gives a simple front door to a complex set of classes.

**Key components**

- **Facade**: `RestaurantFacade`: offers one method such as `placeOrder()`.
- **Subsystem classes**: `InventoryService`, `PaymentService`, `KitchenService`, `NotificationService`: do the real work.
- **Client**: the app or waiter code: calls the facade only.

**Examples**

- **Restaurant**: A customer just says "one burger, please". The facade checks stock, takes payment, tells the kitchen and sends a notification.
- **Common software**: A `VideoConverter.convert(file, format)` method that hides codecs, buffers and audio mixing.

**Pros**

- Client code becomes short and easy to read.
- Client is kept away from subsystem details.
- Subsystems can change without touching clients.

**Cons**

- The facade can grow into a "god object" that does too much.
- Can hide useful features if clients are forced to use only the facade.

---


Now imagine your restaurant's kitchen has become complicated.

To place an order you need:

```text
Inventory
Payment
Kitchen
Chef
Table
Notification
Delivery
Receipt
```

Without a Facade:

```java
inventory.reserve();
payment.authorize();
kitchen.createTicket();
chef.prepare();
receipt.generate();
notification.send();
```

The customer-facing application knows too much.

---

## Restaurant story

Instead of asking the customer to coordinate the entire kitchen:

> "Please reserve ingredients, authorize payment, create a kitchen ticket, notify the chef, generate receipt..."

You give them one counter:

```text
RestaurantService
       │
       ├── Inventory
       ├── Payment
       ├── Kitchen
       ├── Receipt
       └── Notification
```

The counter is the **Facade**.

---

## Code

```java
class InventoryService {

    public void reserve(String item) {
        System.out.println("Inventory reserved: " + item);
    }
}
```

```java
class PaymentService {

    public void charge(double amount) {
        System.out.println("Payment charged: " + amount);
    }
}
```

```java
class KitchenService {

    public void prepare(String item) {
        System.out.println("Kitchen preparing: " + item);
    }
}
```

```java
class NotificationService {

    public void send(String message) {
        System.out.println("Notification: " + message);
    }
}
```

Facade:

```java
class RestaurantFacade {

    private final InventoryService inventory;
    private final PaymentService payment;
    private final KitchenService kitchen;
    private final NotificationService notification;

    public RestaurantFacade(
        InventoryService inventory,
        PaymentService payment,
        KitchenService kitchen,
        NotificationService notification
    ) {
        this.inventory = inventory;
        this.payment = payment;
        this.kitchen = kitchen;
        this.notification = notification;
    }

    public void placeOrder(String item, double amount) {

        inventory.reserve(item);

        payment.charge(amount);

        kitchen.prepare(item);

        notification.send(
            "Order confirmed: " + item
        );
    }
}
```

Client:

```java
public class Main {

    public static void main(String[] args) {

        RestaurantFacade restaurant =
            new RestaurantFacade(
                new InventoryService(),
                new PaymentService(),
                new KitchenService(),
                new NotificationService()
            );

        restaurant.placeOrder("Pizza", 800);
    }
}
```

The client now knows:

```java
restaurant.placeOrder(...)
```

instead of knowing the whole subsystem.

---

## Key idea

Facade doesn't necessarily **replace** the underlying classes.

It gives you a:

> **simple entry point into a complicated subsystem.**

---

### Facade vs Adapter

Very important:

```text
Adapter:
    makes incompatible interfaces compatible

Facade:
    makes a complicated subsystem easier to use
```

---

# 8. Decorator Pattern

**At a glance**

Decorator adds new behavior to an object at runtime by wrapping it in another object with the same interface.

**Key components**

- **Component**: `Coffee`: the common interface (`cost()`, `description()`).
- **Concrete Component**: `SimpleCoffee`: the plain coffee.
- **Base Decorator**: `CoffeeDecorator`: holds a `Coffee` and passes calls to it.
- **Concrete Decorators**: `MilkDecorator`, `CaramelDecorator`: add their own price and name.

**Examples**

- **Restaurant**: A plain coffee wrapped with milk, then caramel. Each wrapper adds to the price. No `CoffeeWithMilkAndCaramel` class is needed.
- **Common software**: Java I/O: `new BufferedReader(new InputStreamReader(new FileInputStream(f)))`.

**Pros**

- Mix and match extras at runtime.
- Avoids a class explosion of every combination.
- Each decorator does one job (Single Responsibility).

**Cons**

- Many small wrapper objects can be hard to debug.
- Order of wrapping can matter and is easy to get wrong.

---


Now suppose your restaurant sells coffee.

Basic:

```text
Coffee
```

Customer can add:

```text
Milk
Sugar
Whipped Cream
Caramel
```

The naive approach is inheritance.

```text
Coffee
 ├── MilkCoffee
 ├── SugarCoffee
 ├── MilkSugarCoffee
 ├── MilkSugarCaramelCoffee
 ├── MilkWhippedCreamCoffee
 ...
```

This becomes a class explosion.

---

## Decorator solution

Instead:

```text
Coffee
  ↓
Milk
  ↓
Caramel
  ↓
WhippedCream
```

Each wrapper adds behavior.

---

## Code

```java
interface Coffee {

    double cost();

    String description();
}
```

Base object:

```java
class SimpleCoffee implements Coffee {

    @Override
    public double cost() {
        return 100;
    }

    @Override
    public String description() {
        return "Coffee";
    }
}
```

Decorator base:

```java
abstract class CoffeeDecorator implements Coffee {

    protected final Coffee coffee;

    protected CoffeeDecorator(Coffee coffee) {
        this.coffee = coffee;
    }
}
```

Milk:

```java
class MilkDecorator extends CoffeeDecorator {

    public MilkDecorator(Coffee coffee) {
        super(coffee);
    }

    @Override
    public double cost() {
        return coffee.cost() + 20;
    }

    @Override
    public String description() {
        return coffee.description() + ", Milk";
    }
}
```

Caramel:

```java
class CaramelDecorator extends CoffeeDecorator {

    public CaramelDecorator(Coffee coffee) {
        super(coffee);
    }

    @Override
    public double cost() {
        return coffee.cost() + 30;
    }

    @Override
    public String description() {
        return coffee.description() + ", Caramel";
    }
}
```

Usage:

```java
public class Main {

    public static void main(String[] args) {

        Coffee coffee =
            new SimpleCoffee();

        coffee =
            new MilkDecorator(coffee);

        coffee =
            new CaramelDecorator(coffee);

        System.out.println(coffee.description());
        System.out.println(coffee.cost());
    }
}
```

Output:

```text
Coffee, Milk, Caramel
150.0
```

---

## Why this is powerful

You can dynamically compose behavior:

```java
new MilkDecorator(
    new CaramelDecorator(
        new SimpleCoffee()
    )
);
```

No new class is required for:

```text
Milk + Caramel
```

or:

```text
Milk + WhippedCream
```

or:

```text
Caramel + WhippedCream + Milk
```

---

## Real-world Java example

Java I/O is famous for decorator-style composition:

```java
InputStream
    ↓
BufferedInputStream
    ↓
GZIPInputStream
```

Each layer adds behavior.

---

# 9. Composite Pattern

**At a glance**

Composite lets you treat single objects and groups of objects the same way by putting them in a tree.

**Key components**

- **Component**: `MenuItem`: the common interface (for example `print()` or `getPrice()`).
- **Leaf**: `FoodItem`: a single dish with no children.
- **Composite**: `MenuGroup`: holds a list of `MenuItem`s, which can be dishes or other groups.
- **Client**: the menu printer: calls `print()` on the top group only.

**Examples**

- **Restaurant**: The full menu holds "Lunch" and "Drinks"; "Lunch" holds dishes. Printing the whole menu and printing one dish use the same call.
- **Common software**: Folders and files in a file system, or nested panels and buttons in a UI.

**Pros**

- Client code does not need `if (isGroup)` checks.
- Easy to add new kinds of items.
- Natural fit for tree data.

**Cons**

- The common interface can become too general (a leaf may have an `add()` it cannot support).
- Harder to limit what a group may contain.

---


Restaurant story:

A restaurant menu can contain:

```text
Pizza
Burger
Drink
```

But it can also contain groups:

```text
Combo Meal
 ├── Burger
 ├── Fries
 └── Drink
```

And:

```text
Family Meal
 ├── Combo Meal
 │    ├── Burger
 │    ├── Fries
 │    └── Drink
 ├── Pizza
 └── Drinks
```

The important requirement is:

> Treat an individual item and a group of items in the same way.

---

## Common interface

```java
interface MenuItem {

    String getName();

    double getPrice();

    void print();
}
```

Leaf:

```java
class FoodItem implements MenuItem {

    private final String name;
    private final double price;

    public FoodItem(String name, double price) {
        this.name = name;
        this.price = price;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public double getPrice() {
        return price;
    }

    @Override
    public void print() {
        System.out.println(name + " - " + price);
    }
}
```

Composite:

```java
class MenuGroup implements MenuItem {

    private final String name;
    private final List<MenuItem> children = new ArrayList<>();

    public MenuGroup(String name) {
        this.name = name;
    }

    public void add(MenuItem item) {
        children.add(item);
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public double getPrice() {

        return children.stream()
            .mapToDouble(MenuItem::getPrice)
            .sum();
    }

    @Override
    public void print() {

        System.out.println(name);

        for (MenuItem item : children) {
            item.print();
        }
    }
}
```

Usage:

```java
MenuItem burger =
    new FoodItem("Burger", 300);

MenuItem fries =
    new FoodItem("Fries", 100);

MenuGroup combo =
    new MenuGroup("Combo Meal");

combo.add(burger);
combo.add(fries);

combo.print();

System.out.println(combo.getPrice());
```

---

## Structure

```text
             MenuItem
             /      \
            /        \
       FoodItem    MenuGroup
                     |
              ┌──────┼──────┐
              ↓      ↓      ↓
            Food   Food    Food
```

That's the heart of Composite:

> **part and whole have the same interface.**

---

## Real-world examples

Composite appears whenever you have trees:

```text
File
 ├── File
 ├── Folder
 │    ├── File
 │    └── Folder
```

UI:

```text
Component
 ├── Button
 ├── TextBox
 └── Panel
      ├── Button
      └── TextBox
```

Organization:

```text
Employee
 ├── Developer
 ├── Developer
 └── Manager
      ├── Developer
      └── Developer
```

---

# 10. Proxy Pattern

**At a glance**

Proxy puts a stand-in object in front of a real one to control access to it.

**Key components**

- **Subject**: `Kitchen`: the interface both share.
- **Real Subject**: `RealKitchen`: does the real cooking.
- **Proxy**: `KitchenProxy`: checks, delays, caches or logs before passing the call on.
- **Client**: the waiter code: uses `Kitchen` and cannot tell the proxy apart.

**Examples**

- **Restaurant**: Customers cannot walk into the kitchen. The proxy checks the order (for example, is the kitchen open?) and only then forwards it.
- **Common software**: A lazy-loading image proxy, an access-control check before a service, or a remote API stub.

**Pros**

- Adds control (security, caching, lazy loading) without changing the real class.
- Client code stays the same.

**Cons**

- One more layer, so calls can be a bit slower.
- Can hide delays that the client should know about.

---


Now imagine your restaurant has an expensive service:

```java
RealKitchenSystem
```

Creating it requires:

```text
network connection
authentication
large configuration
remote connection
```

But perhaps the customer isn't even authorized.

Instead of directly accessing it:

```text
Client
   ↓
RealKitchenSystem
```

we put a representative in front:

```text
Client
   ↓
KitchenProxy
   ↓
RealKitchenSystem
```

The Proxy controls access.

---

## Code

```java
interface Kitchen {

    void prepareOrder(String order);
}
```

Real object:

```java
class RealKitchen implements Kitchen {

    public RealKitchen() {
        connectToKitchenSystem();
    }

    private void connectToKitchenSystem() {
        System.out.println(
            "Connecting to expensive kitchen system..."
        );
    }

    @Override
    public void prepareOrder(String order) {
        System.out.println(
            "Preparing: " + order
        );
    }
}
```

Proxy:

```java
class KitchenProxy implements Kitchen {

    private RealKitchen realKitchen;

    @Override
    public void prepareOrder(String order) {

        if (!isAuthorized()) {
            throw new SecurityException(
                "Not authorized"
            );
        }

        if (realKitchen == null) {
            realKitchen = new RealKitchen();
        }

        realKitchen.prepareOrder(order);
    }

    private boolean isAuthorized() {
        return true;
    }
}
```

Client:

```java
Kitchen kitchen =
    new KitchenProxy();

kitchen.prepareOrder("Pizza");
```

---

## What did Proxy give us?

The proxy can add:

### Access control

```text
Is user authorized?
```

### Lazy loading

```text
Don't create expensive object until necessary.
```

### Logging

```text
Who called this?
```

### Caching

```text
Do we already have this result?
```

### Remote access

```text
Local object
    ↓
Proxy
    ↓
Remote service
```

---

## Real-world examples

Common proxy uses include:

```text
Spring AOP
Lazy-loading ORM entities
Remote service clients
Security proxies
Caching proxies
Logging proxies
```

---

# 11. Bridge Pattern

**At a glance**

Bridge splits a class into two separate hierarchies, what it is and how it does it, so both can change on their own.

**Key components**

- **Abstraction**: abstract `Notification`: holds a `NotificationSender`.
- **Refined Abstraction**: `OrderNotification`: one kind of message.
- **Implementor**: `NotificationSender`: the interface for sending.
- **Concrete Implementors**: `EmailSender`, `SmsSender`: the real channels.

**Examples**

- **Restaurant**: "Order ready" can go by email or SMS. New message types and new channels are added on separate sides, with no `OrderEmailNotification`, `OrderSmsNotification`, and so on.
- **Common software**: A remote control (abstraction) that works with any TV brand (implementation), or JDBC drivers under one `Connection` API.

**Pros**

- Avoids a class for every type-and-channel combination.
- Each side can grow on its own (Open/Closed).
- Implementation can be chosen at runtime.

**Cons**

- Harder to understand at first glance.
- Overkill if there is only one implementation.

---


This one is harder, so let's slow down.

Suppose a restaurant has different notification types:

```text
Order Notification
Payment Notification
Delivery Notification
```

And each can be delivered through:

```text
Email
SMS
Push
```

Naive inheritance produces:

```text
EmailOrderNotification
SMSOrderNotification
PushOrderNotification

EmailPaymentNotification
SMSPaymentNotification
PushPaymentNotification

EmailDeliveryNotification
SMSDeliveryNotification
PushDeliveryNotification
```

This becomes:

```text
3 × 3 = 9 classes
```

Then:

```text
5 notification types × 5 channels
= 25 classes
```

---

## Bridge

Separate the two dimensions.

```text
Notification
     │
     │ uses
     ▼
NotificationSender
     │
 ┌───┼────┐
 ↓   ↓    ↓
Email SMS Push
```

---

## Implementation

```java
interface NotificationSender {

    void send(String message);
}
```

Implementations:

```java
class EmailSender implements NotificationSender {

    @Override
    public void send(String message) {
        System.out.println(
            "EMAIL: " + message
        );
    }
}
```

```java
class SmsSender implements NotificationSender {

    @Override
    public void send(String message) {
        System.out.println(
            "SMS: " + message
        );
    }
}
```

Abstraction:

```java
abstract class Notification {

    protected final NotificationSender sender;

    protected Notification(
        NotificationSender sender
    ) {
        this.sender = sender;
    }

    public abstract void send();
}
```

Concrete abstraction:

```java
class OrderNotification
        extends Notification {

    private final String orderId;

    public OrderNotification(
        String orderId,
        NotificationSender sender
    ) {
        super(sender);
        this.orderId = orderId;
    }

    @Override
    public void send() {
        sender.send(
            "Order " + orderId + " confirmed"
        );
    }
}
```

Usage:

```java
Notification notification =
    new OrderNotification(
        "ORD-100",
        new EmailSender()
    );

notification.send();
```

Or:

```java
Notification notification =
    new OrderNotification(
        "ORD-100",
        new SmsSender()
    );
```

Now adding:

```text
WhatsAppSender
```

doesn't require:

```text
WhatsAppOrderNotification
WhatsAppPaymentNotification
WhatsAppDeliveryNotification
```

---

## Bridge's core idea

> **Separate two dimensions that can change independently.**

This is one of the most important structural patterns for architecture.

---

# 12. Flyweight Pattern

**At a glance**

Flyweight saves memory by sharing the parts of objects that many of them have in common.

**Key components**

- **Flyweight**: final `CustomerProfile`: the shared, immutable part (intrinsic state: country, language, currency).
- **Flyweight Factory**: `CustomerProfileFactory`: returns an existing profile or creates it once.
- **Extrinsic state**: data unique to each customer, such as name or order list, kept outside the flyweight.
- **Client**: customer objects: hold a reference to a shared profile.

**Examples**

- **Restaurant**: 100,000 customers from Bangladesh all point to one "Bangladesh, Bengali, BDT" profile instead of 100,000 copies.
- **Common software**: Characters in a text editor sharing font data, or trees in a game sharing one mesh and texture.

**Pros**

- Large memory savings when many objects repeat the same data.
- Shared objects are immutable, so they are safe to share.

**Cons**

- Code is more complex: state is split into two parts.
- Gains are small unless you have very many similar objects.

---


Imagine a restaurant with:

```text
100,000 customers
```

Each customer has:

```text
country
language
currency
membership type
```

But many customers share the same data.

Instead of storing duplicate immutable objects:

```text
Customer 1 → BangladeshProfile
Customer 2 → BangladeshProfile
Customer 3 → BangladeshProfile
...
```

we can share the common object.

---

## Intrinsic vs extrinsic state

This is the key concept.

### Intrinsic

Shared:

```text
Country = Bangladesh
Language = Bengali
Currency = BDT
```

### Extrinsic

Unique:

```text
Customer ID
Name
Address
```

Flyweight stores the reusable intrinsic part.

---

## Code

```java
final class CustomerProfile {

    private final String country;
    private final String language;
    private final String currency;

    public CustomerProfile(
        String country,
        String language,
        String currency
    ) {
        this.country = country;
        this.language = language;
        this.currency = currency;
    }

    public void displayFor(String customerId) {

        System.out.println(
            customerId
            + " | "
            + country
            + " | "
            + language
            + " | "
            + currency
        );
    }
}
```

Factory:

```java
class CustomerProfileFactory {

    private final Map<String, CustomerProfile>
        profiles = new HashMap<>();

    public CustomerProfile getProfile(
        String country,
        String language,
        String currency
    ) {

        String key =
            country + ":" +
            language + ":" +
            currency;

        return profiles.computeIfAbsent(
            key,
            k -> new CustomerProfile(
                country,
                language,
                currency
            )
        );
    }
}
```

Usage:

```java
CustomerProfileFactory factory =
    new CustomerProfileFactory();

CustomerProfile p1 =
    factory.getProfile(
        "Bangladesh",
        "Bengali",
        "BDT"
    );

CustomerProfile p2 =
    factory.getProfile(
        "Bangladesh",
        "Bengali",
        "BDT"
    );

System.out.println(p1 == p2);
```

Output:

```text
true
```

Same shared object.

---

# Structural patterns — now connect them

You can now see a major difference:

| Pattern   | Main question                                                    |
| --------- | ---------------------------------------------------------------- |
| Adapter   | How do I make incompatible things work together?                 |
| Facade    | How do I simplify a complicated subsystem?                       |
| Decorator | How do I add behavior dynamically?                               |
| Composite | How do I treat individual objects and groups uniformly?          |
| Proxy     | How do I control access to an object?                            |
| Bridge    | How do I separate two independently changing dimensions?         |
| Flyweight | How do I share common state to reduce memory/object duplication? |

---

# PART III — BEHAVIORAL PATTERNS

<img width="1024" height="559" alt="image" src="https://github.com/user-attachments/assets/05b103ef-438d-4e9e-8be0-1bde0a1352be" />

Now we move from:

> **How objects are structured**

to:

> **How objects communicate and behave.**

---

# 13. Strategy Pattern

**At a glance**

Strategy puts each version of an algorithm in its own class so you can swap them at runtime.

**Key components**

- **Strategy**: `DeliveryStrategy`: the common interface with `calculateFee(distance)`.
- **Concrete Strategies**: `NormalDelivery`, `ExpressDelivery`: different ways to deliver.
- **Context**: `DeliveryService`: holds a strategy and uses it.

**Examples**

- **Restaurant**: The customer picks normal or express delivery. The service just calls `calculateFee()` on whichever was chosen.
- **Common software**: Payment methods in a checkout, or sorting with different `Comparator`s in Java.

**Pros**

- Removes long `if/else` or `switch` blocks.
- New strategies are added without touching the context (Open/Closed).
- Each algorithm can be tested on its own.

**Cons**

- The client must know the strategies to choose one.
- Overkill for one or two simple variations.

---


This is one of the most useful patterns in everyday application development.

Restaurant:

```text
Calculate delivery fee
```

Different strategies:

```text
NormalDelivery
ExpressDelivery
FreeDelivery
DistanceBasedDelivery
```

Instead of:

```java
if (type.equals("normal")) {
    ...
}
else if (type.equals("express")) {
    ...
}
else if ...
```

we encapsulate the algorithm.

---

## Interface

```java
interface DeliveryStrategy {

    double calculateFee(double distance);
}
```

Implementations:

```java
class NormalDelivery
        implements DeliveryStrategy {

    @Override
    public double calculateFee(double distance) {
        return distance * 20;
    }
}
```

```java
class ExpressDelivery
        implements DeliveryStrategy {

    @Override
    public double calculateFee(double distance) {
        return distance * 35;
    }
}
```

Context:

```java
class DeliveryService {

    private final DeliveryStrategy strategy;

    public DeliveryService(
        DeliveryStrategy strategy
    ) {
        this.strategy = strategy;
    }

    public double calculate(double distance) {
        return strategy.calculateFee(distance);
    }
}
```

Usage:

```java
DeliveryService service =
    new DeliveryService(
        new ExpressDelivery()
    );

System.out.println(
    service.calculate(10)
);
```

---

## The important concept

Strategy means:

> **The algorithm can vary independently from the object using it.**

This is why Strategy and Factory are often used together.

Factory:

```text
Which strategy should I create?
```

Strategy:

```text
How should the operation be performed?
```

---

# 14. Observer Pattern

**At a glance**

Observer lets one object notify a list of other objects automatically when its state changes.

**Key components**

- **Subject**: `Order`: keeps a list of observers and notifies them.
- **Observer**: `OrderObserver`: the interface with an `update()` method.
- **Concrete Observers**: `Kitchen`, `Customer`: react when the order changes.

**Examples**

- **Restaurant**: When an order is placed, the kitchen starts cooking and the customer gets a message, without `Order` knowing their details.
- **Common software**: Event listeners on a UI button, or subscribers to a newsletter or message topic.

**Pros**

- Subject and observers are loosely coupled.
- New observers can be added at runtime (Open/Closed).

**Cons**

- Notification order is not guaranteed.
- Forgotten subscriptions can cause memory leaks.
- Long chains of updates can be hard to trace.

---


Restaurant example:

When an order status changes:

```text
Order confirmed
       ↓
 ┌─────┼───────────┐
 ↓     ↓           ↓
Kitchen Customer  Delivery
```

The order shouldn't have to know every consumer.

---

## Observer interface

```java
interface OrderObserver {

    void update(String status);
}
```

Observers:

```java
class Kitchen implements OrderObserver {

    @Override
    public void update(String status) {
        System.out.println(
            "Kitchen received: " + status
        );
    }
}
```

```java
class Customer implements OrderObserver {

    @Override
    public void update(String status) {
        System.out.println(
            "Customer received: " + status
        );
    }
}
```

Subject:

```java
class Order {

    private final List<OrderObserver>
        observers = new ArrayList<>();

    public void subscribe(OrderObserver observer) {
        observers.add(observer);
    }

    public void changeStatus(String status) {

        System.out.println(
            "Order status: " + status
        );

        for (OrderObserver observer : observers) {
            observer.update(status);
        }
    }
}
```

Usage:

```java
Order order = new Order();

order.subscribe(new Kitchen());
order.subscribe(new Customer());

order.changeStatus("READY");
```

Output:

```text
Order status: READY
Kitchen received: READY
Customer received: READY
```

---

## Real-world examples

Observer-like mechanisms appear everywhere:

```text
UI events
Event listeners
Domain events
Messaging systems
Pub/Sub
Reactive streams
```

Spring application events are another example of the same broad idea.

---

# 15. Command Pattern

**At a glance**

Command turns a request into an object, so it can be stored, queued, logged or undone.

**Key components**

- **Command**: `Command`: interface with `execute()`.
- **Concrete Command**: `PrepareOrderCommand`: knows what to ask the kitchen to do.
- **Receiver**: `Kitchen`: does the actual work.
- **Invoker**: `Waiter`: takes commands and runs them, without knowing how cooking works.

**Examples**

- **Restaurant**: The waiter writes the order on a ticket (the command) and hands it in. The ticket can wait in a queue, be cancelled or be repeated.
- **Common software**: Undo/redo in a text editor, or menu buttons and keyboard shortcuts that trigger the same action.

**Pros**

- Separates who asks from who does the work.
- Supports undo, queues and logs.
- Commands can be combined into macros.

**Cons**

- One class per action, so many small classes.

---


Suppose restaurant operations include:

```text
PlaceOrder
CancelOrder
RefundOrder
PrintReceipt
```

Instead of directly executing everything, represent an operation as an object.

```text
Command
   ↓
PlaceOrderCommand
CancelOrderCommand
RefundCommand
```

---

## Code

```java
interface Command {

    void execute();
}
```

Receiver:

```java
class Kitchen {

    public void prepare(String item) {
        System.out.println(
            "Preparing " + item
        );
    }
}
```

Command:

```java
class PrepareOrderCommand
        implements Command {

    private final Kitchen kitchen;
    private final String item;

    public PrepareOrderCommand(
        Kitchen kitchen,
        String item
    ) {
        this.kitchen = kitchen;
        this.item = item;
    }

    @Override
    public void execute() {
        kitchen.prepare(item);
    }
}
```

Invoker:

```java
class Waiter {

    public void takeOrder(Command command) {
        command.execute();
    }
}
```

Usage:

```java
Kitchen kitchen = new Kitchen();

Command command =
    new PrepareOrderCommand(
        kitchen,
        "Pizza"
    );

Waiter waiter = new Waiter();

waiter.takeOrder(command);
```

---

## Why make an operation an object?

Because now commands can be:

```text
stored
queued
logged
retried
undone
scheduled
serialized
```

This becomes extremely useful in:

```text
Job queues
GUI actions
Undo/redo
Transactions
Message processing
CQRS
```

---

# 16. Template Method

**At a glance**

Template Method defines the steps of an algorithm in a parent class and lets subclasses fill in some of the steps.

**Key components**

- **Abstract Class**: abstract `Meal`: holds the fixed `prepare()` flow and the abstract steps.
- **Concrete Classes**: `ItalianMeal`, `BengaliMeal`: fill in the steps such as cooking and serving.

**Examples**

- **Restaurant**: Every meal follows prepare ingredients, cook, serve. Italian and Bengali meals only change the cooking and serving steps.
- **Common software**: A data importer whose `run()` always does open, read, parse, close, while subclasses define `parse()` for CSV or JSON.

**Pros**

- The common flow is written once.
- Subclasses cannot break the order of steps.

**Cons**

- Based on inheritance, so it is less flexible than Strategy.
- Hard to follow when the template has many steps and hooks.

---


Restaurant has a standard meal preparation process:

```text
1. Prepare ingredients
2. Cook
3. Plate
4. Serve
```

The overall algorithm is fixed.

But the details vary.

```text
ItalianMeal
BengaliMeal
IndianMeal
```

---

## Template

```java
abstract class Meal {

    public final void prepareMeal() {

        prepareIngredients();

        cook();

        plate();

        serve();
    }

    protected abstract void prepareIngredients();

    protected abstract void cook();

    protected void plate() {
        System.out.println("Plating meal");
    }

    protected void serve() {
        System.out.println("Serving meal");
    }
}
```

Italian:

```java
class ItalianMeal extends Meal {

    @Override
    protected void prepareIngredients() {
        System.out.println(
            "Preparing pasta ingredients"
        );
    }

    @Override
    protected void cook() {
        System.out.println(
            "Cooking pasta"
        );
    }
}
```

Bengali:

```java
class BengaliMeal extends Meal {

    @Override
    protected void prepareIngredients() {
        System.out.println(
            "Preparing rice and spices"
        );
    }

    @Override
    protected void cook() {
        System.out.println(
            "Cooking rice and curry"
        );
    }
}
```

The client:

```java
Meal meal = new BengaliMeal();

meal.prepareMeal();
```

---

## Core idea

Template Method says:

> **The parent controls the algorithm; subclasses customize selected steps.**

This is different from Factory Method.

Factory Method:

```text
Which object do I create?
```

Template Method:

```text
How does the overall algorithm execute?
```

---

# 17. State Pattern

**At a glance**

State lets an object change its behavior when its internal state changes, by moving each state's behavior into its own class.

**Key components**

- **Context**: `Order`: holds the current state and passes calls to it.
- **State**: `OrderState`: the interface for actions such as `next()`.
- **Concrete States**: `NewState`, then `PaidState`, `PreparingState`, `ReadyState`: each knows what to do and which state comes next.

**Examples**

- **Restaurant**: An order moves from new to paid to preparing to ready to delivered. Each stage decides what happens next, with no big `if (status == ...)` blocks.
- **Common software**: A media player (playing, paused, stopped), or a TCP connection (listening, established, closed).

**Pros**

- Removes large state `switch` blocks.
- Each state's rules live in one class.
- New states are easy to add.

**Cons**

- Many small classes for simple state machines.
- Transitions are spread across classes.

---


Imagine an order.

Its behavior depends on state:

```text
NEW
 ↓
PAID
 ↓
PREPARING
 ↓
READY
 ↓
DELIVERED
```

Without State:

```java
if (status == NEW) ...
else if (status == PAID) ...
else if (status == PREPARING) ...
```

and these conditions appear everywhere.

State moves behavior into state objects.

```text
Order
  ↓
Current State
  ↓
PaidState
PreparingState
ReadyState
```

For example:

```java
interface OrderState {

    void next(Order order);

    void cancel(Order order);
}
```

```java
class NewState implements OrderState {

    @Override
    public void next(Order order) {
        order.setState(new PaidState());
    }

    @Override
    public void cancel(Order order) {
        System.out.println("Order cancelled");
    }
}
```

Context:

```java
class Order {

    private OrderState state =
        new NewState();

    public void setState(OrderState state) {
        this.state = state;
    }

    public void next() {
        state.next(this);
    }

    public void cancel() {
        state.cancel(this);
    }
}
```

---

## State vs Strategy

They look very similar.

### Strategy

Usually selected from outside:

```text
DeliveryService
     ↓
Strategy
```

The algorithm is chosen.

### State

Usually changes from inside:

```text
Order
 ↓
NewState
 ↓
PaidState
 ↓
PreparingState
```

The object's behavior changes because its **state changes**.

---

# 18. Chain of Responsibility

**At a glance**

Chain of Responsibility passes a request along a chain of handlers until one of them handles it.

**Key components**

- **Handler**: abstract `ComplaintHandler`: holds the next handler and a `handle()` method.
- **Concrete Handlers**: `Waiter`, `Manager`: each handles what it can and passes the rest on.
- **Client**: the customer complaint code: sends to the first handler only.

**Examples**

- **Restaurant**: A small complaint is fixed by the waiter. A bigger one goes up to the manager.
- **Common software**: Servlet filters or middleware in a web framework, or support tickets moving from level 1 to level 2.

**Pros**

- The sender does not need to know who handles the request.
- Handlers can be added or reordered easily.

**Cons**

- A request may reach the end with no one handling it.
- Long chains are hard to debug.

---


Restaurant example:

A customer complaint goes through:

```text
Waiter
  ↓
Supervisor
  ↓
Manager
  ↓
General Manager
```

Each person can handle the request or pass it onward.

```text
Request
   ↓
Waiter
   ↓
Supervisor
   ↓
Manager
```

---

## Code

```java
abstract class ComplaintHandler {

    private ComplaintHandler next;

    public ComplaintHandler setNext(
        ComplaintHandler next
    ) {
        this.next = next;
        return next;
    }

    public void handle(int severity) {

        if (canHandle(severity)) {
            process(severity);
        }
        else if (next != null) {
            next.handle(severity);
        }
        else {
            System.out.println(
                "No one can handle it"
            );
        }
    }

    protected abstract boolean canHandle(
        int severity
    );

    protected abstract void process(
        int severity
    );
}
```

Waiter:

```java
class Waiter extends ComplaintHandler {

    protected boolean canHandle(int severity) {
        return severity <= 2;
    }

    protected void process(int severity) {
        System.out.println(
            "Waiter handled complaint"
        );
    }
}
```

Manager:

```java
class Manager extends ComplaintHandler {

    protected boolean canHandle(int severity) {
        return severity <= 5;
    }

    protected void process(int severity) {
        System.out.println(
            "Manager handled complaint"
        );
    }
}
```

Usage:

```java
ComplaintHandler waiter =
    new Waiter();

ComplaintHandler manager =
    new Manager();

waiter.setNext(manager);

waiter.handle(4);
```

---

## Real-world examples

```text
HTTP middleware
Authentication filters
Authorization pipeline
Logging pipeline
Exception handling
Request processing
Approval workflows
```

A web request often looks like:

```text
Request
 ↓
Logging
 ↓
Authentication
 ↓
Authorization
 ↓
Validation
 ↓
Controller
```

That's conceptually very close to Chain of Responsibility.

---

# 19. Iterator

**At a glance**

Iterator lets you walk through a collection one item at a time without knowing how it is stored.

**Key components**

- **Iterator**: `Iterator<MenuItem>`: has `hasNext()` and `next()`.
- **Aggregate**: the menu: offers `iterator()`.
- **Concrete storage**: `ArrayList`, `LinkedList`, a tree or a database cursor: hidden from the client.
- **Client**: the code that prints the menu: just loops.

**Examples**

- **Restaurant**: The customer reads the menu item by item. They do not care whether it is stored as a list, a tree or rows in a database.
- **Common software**: Java's `for-each` loop over any `Iterable`, or paging through search results.

**Pros**

- Client code does not depend on how data is stored.
- Several traversals can run at the same time.
- Storage can change without changing client loops.

**Cons**

- Overkill for a simple array.
- Changing the collection while iterating can fail.

---


Suppose your restaurant menu internally uses:

```java
List<MenuItem>
```

The customer shouldn't need to know whether it is stored as:

```text
ArrayList
LinkedList
Tree
Database cursor
```

They simply iterate.

```java
Iterator<MenuItem> iterator =
    menu.iterator();

while (iterator.hasNext()) {

    MenuItem item =
        iterator.next();

    System.out.println(
        item.getName()
    );
}
```

The pattern separates:

```text
Collection storage
```

from:

```text
Traversal
```

Java's:

```java
Iterator<T>
```

is literally an implementation of this pattern.

---

# 20. Mediator

**At a glance**

Mediator makes objects talk through one central object instead of talking to each other directly.

**Key components**

- **Mediator**: `RestaurantMediator`: the interface with `notify(sender, event)`.
- **Concrete Mediator**: `RestaurantCoordinator`: knows all parts and decides who reacts to what.
- **Colleagues**: chef, waiter, payment and delivery: send events to the mediator only.

**Examples**

- **Restaurant**: The chef does not call the waiter, payment and delivery one by one. They tell the coordinator "food ready", and the coordinator tells the right people.
- **Common software**: An air traffic control tower, or a chat room that passes messages between users.

**Pros**

- Removes a tangled many-to-many web of links.
- Parts can be reused and changed on their own.

**Cons**

- The mediator can become a large "god object".

---


Imagine every restaurant component directly communicates with every other component:

```text
Chef ↔ Waiter
Chef ↔ Payment
Chef ↔ Delivery
Waiter ↔ Payment
Waiter ↔ Delivery
Payment ↔ Delivery
...
```

Communication becomes a tangled network.

Mediator introduces a central coordinator:

```text
             RestaurantMediator
             /       |       \
            /        |        \
        Chef       Waiter    Payment
```

Components communicate through the mediator.

```java
interface RestaurantMediator {

    void notify(
        Object sender,
        String event
    );
}
```

Then:

```java
class RestaurantCoordinator
        implements RestaurantMediator {

    @Override
    public void notify(
        Object sender,
        String event
    ) {

        if (event.equals("ORDER_READY")) {
            System.out.println(
                "Notify waiter"
            );
        }
    }
}
```

The key idea:

> **Centralize complex communication.**

This appears in:

```text
UI coordination
Workflow engines
Chat rooms
Air traffic coordination
Application orchestration
```

---

# 21. Memento

**At a glance**

Memento saves a snapshot of an object's state so it can be restored later, without exposing its insides.

**Key components**

- **Originator**: `MenuEditor`: creates snapshots and restores from them.
- **Memento**: record `MenuMemento`: an immutable snapshot of the menu.
- **Caretaker**: the code that keeps the history list: stores mementos but never reads their contents.

**Examples**

- **Restaurant**: The manager edits the menu, does not like the change and rolls back to yesterday's version.
- **Common software**: Undo in a text editor, or save points in a game.

**Pros**

- Undo without breaking encapsulation.
- The originator's code stays simple.

**Cons**

- Snapshots can use a lot of memory.
- The caretaker must clean up old mementos.

---


Suppose a restaurant manager edits a menu:

```text
Menu v1
 ↓
Menu v2
 ↓
Menu v3
```

They want:

> Undo my last change.

Memento stores a snapshot.

```text
Menu
 ↓
createSnapshot()
 ↓
Memento
```

Later:

```text
Menu.restore(memento)
```

---

## Simple implementation

```java
record MenuMemento(String content) {
}
```

Originator:

```java
class MenuEditor {

    private String content;

    public void setContent(String content) {
        this.content = content;
    }

    public MenuMemento save() {
        return new MenuMemento(content);
    }

    public void restore(MenuMemento memento) {
        this.content = memento.content();
    }

    public void show() {
        System.out.println(content);
    }
}
```

Usage:

```java
MenuEditor editor =
    new MenuEditor();

editor.setContent("Pizza, Burger");

MenuMemento backup =
    editor.save();

editor.setContent("Pizza only");

editor.restore(backup);

editor.show();
```

Output:

```text
Pizza, Burger
```

---

## Real-world uses

```text
Undo/redo
Draft versions
Transaction snapshots
Game save points
Document history
```

---

# 22. Visitor

**At a glance**

Visitor lets you add new operations to a set of classes without changing those classes.

**Key components**

- **Visitor**: `FoodVisitor`: one `visit()` method per food type.
- **Concrete Visitor**: `TaxVisitor`: computes tax for each food type.
- **Element**: `Food`: has `accept(visitor)`.
- **Concrete Elements**: `Burger`, `Pizza`: call back the matching `visit()` method.

**Examples**

- **Restaurant**: A tax inspector visits each dish and applies the right tax rule. Later a discount inspector can be added without touching `Burger` or `Pizza`.
- **Common software**: Compilers walking a syntax tree for type checking and code generation, or exporting shapes to different file formats.

**Pros**

- New operations are added without changing element classes (Open/Closed).
- Related logic stays together in one visitor.

**Cons**

- Adding a new element type means changing every visitor.
- Visitors may need access to the elements' private details.

---


Visitor is more advanced.

Suppose restaurant objects are:

```text
Burger
Pizza
Drink
```

Different operations may be performed on them:

```text
Calculate tax
Calculate discount
Generate nutrition report
Export accounting data
```

If we keep adding methods to every food class:

```text
Burger
 ├── tax()
 ├── discount()
 ├── nutrition()
 └── accounting()

Pizza
 ├── tax()
 ├── discount()
 ├── nutrition()
 └── accounting()
```

the domain classes become overloaded.

Visitor moves operations outside the objects.

---

## Visitor

```java
interface FoodVisitor {

    void visit(Burger burger);

    void visit(Pizza pizza);
}
```

Elements:

```java
interface Food {

    void accept(FoodVisitor visitor);
}
```

Burger:

```java
class Burger implements Food {

    @Override
    public void accept(FoodVisitor visitor) {
        visitor.visit(this);
    }
}
```

Pizza:

```java
class Pizza implements Food {

    @Override
    public void accept(FoodVisitor visitor) {
        visitor.visit(this);
    }
}
```

Tax visitor:

```java
class TaxVisitor implements FoodVisitor {

    @Override
    public void visit(Burger burger) {
        System.out.println(
            "Calculate burger tax"
        );
    }

    @Override
    public void visit(Pizza pizza) {
        System.out.println(
            "Calculate pizza tax"
        );
    }
}
```

Usage:

```java
Food food = new Pizza();

food.accept(new TaxVisitor());
```

---

## When Visitor is useful

Particularly when:

```text
Object structure changes rarely
BUT
Operations change frequently
```

Examples:

```text
Compiler AST
Document processing
File-system trees
Static analysis
Reporting
Code analysis
```

It's powerful, but considerably more complex than Strategy or Observer.

---

# 23. Interpreter

**At a glance**

Interpreter represents the rules of a small language as classes, so sentences in that language can be evaluated.

**Key components**

- **Abstract Expression**: `Expression`: has `interpret(context)`.
- **Terminal Expression**: `PriceExpression` or a number such as `500`: the basic parts.
- **Non-terminal Expression**: for example `GreaterThanExpression`: combines other expressions.
- **Context**: `MenuContext`: holds the data the expressions read, such as the item's price.

**Examples**

- **Restaurant**: Staff type `PRICE > 500` or `CATEGORY = PIZZA` to filter the menu. Each part of the query is an object that knows how to evaluate itself.
- **Common software**: Regular expressions, SQL `WHERE` filters, or rule engines with simple conditions.

**Pros**

- Easy to add new rules as new classes.
- The grammar is clear from the class structure.

**Cons**

- Grows hard to manage for large grammars; use a parser generator instead.
- Can be slow for complex expressions.

---


This is the final GoF pattern.

Suppose your restaurant allows a simple query language:

```text
PRICE > 500
```

or:

```text
CATEGORY = PIZZA
```

You need to interpret expressions.

The structure becomes:

```text
Expression
   ├── TerminalExpression
   └── NonTerminalExpression
```

For example:

```text
PRICE > 500
```

can be represented as:

```text
GreaterThanExpression
        │
        ├── PriceExpression
        └── 500
```

A simplified interface:

```java
interface Expression {

    boolean interpret(MenuContext context);
}
```

Context:

```java
class MenuContext {

    private final double price;

    public MenuContext(double price) {
        this.price = price;
    }

    public double getPrice() {
        return price;
    }
}
```

Terminal:

```java
class PriceExpression
        implements Expression {

    @Override
    public boolean interpret(
        MenuContext context
    ) {
        return context.getPrice() > 500;
    }
}
```

Usage:

```java
Expression expression =
    new PriceExpression();

boolean result =
    expression.interpret(
        new MenuContext(800)
    );

System.out.println(result);
```

Output:

```text
true
```

Real interpreters are usually much more sophisticated.

Examples:

```text
SQL-like languages
Rule engines
Expression languages
Configuration languages
DSLs
Parsers
```

---

# COMPLETE MAP — ALL 23 GOF PATTERNS

You now have the whole Gang of Four set:

```text
DESIGN PATTERNS
│
├── CREATIONAL
│   │
│   ├── Singleton
│   ├── Factory Method
│   ├── Abstract Factory
│   ├── Builder
│   └── Prototype
│
├── STRUCTURAL
│   │
│   ├── Adapter
│   ├── Bridge
│   ├── Composite
│   ├── Decorator
│   ├── Facade
│   ├── Flyweight
│   └── Proxy
│
└── BEHAVIORAL
    │
    ├── Chain of Responsibility
    ├── Command
    ├── Interpreter
    ├── Iterator
    ├── Mediator
    ├── Memento
    ├── Observer
    ├── State
    ├── Strategy
    ├── Template Method
    └── Visitor
```

# The most important mental model

Don't memorize 23 names.

Ask **what problem am I actually seeing?**

```text
"I need ONE shared instance."
        ↓
    Singleton


"Which implementation should I create?"
        ↓
    Factory Method


"I need a compatible FAMILY of objects."
        ↓
    Abstract Factory


"This object has too many construction options."
        ↓
    Builder


"I already have an object and want another like it."
        ↓
    Prototype


"These two interfaces don't match."
        ↓
    Adapter


"This subsystem is too complicated for clients."
        ↓
    Facade


"I want to add behavior without creating subclasses for every combination."
        ↓
    Decorator


"Individual objects and groups should look the same."
        ↓
    Composite


"I need to control access to an object."
        ↓
    Proxy


"Two dimensions need to vary independently."
        ↓
    Bridge


"Many objects share the same intrinsic data."
        ↓
    Flyweight


"The algorithm can vary."
        ↓
    Strategy


"Many objects need notification when something changes."
        ↓
    Observer


"I want to represent an operation as an object."
        ↓
    Command


"The algorithm skeleton is fixed but steps vary."
        ↓
    Template Method


"Object behavior changes according to its state."
        ↓
    State


"Let one handler process it, otherwise pass it along."
        ↓
    Chain of Responsibility


"I need controlled traversal of a collection."
        ↓
    Iterator


"Many objects communicate in a tangled way."
        ↓
    Mediator


"I need snapshots/undo."
        ↓
    Memento


"I need to add many operations to a stable object structure."
        ↓
    Visitor


"I need to evaluate a language/expression."
        ↓
    Interpreter
```

## One more distinction that will make all 23 easier

Think of them as solving five different architectural questions:

```text
             OBJECT DESIGN
                  │
       ┌──────────┼──────────┐
       │          │          │
    CREATE      STRUCTURE  BEHAVIOR
       │          │          │
       │          │          │
       ▼          ▼          ▼
   Creational  Structural  Behavioral
```

Then within those:

```text
CREATIONAL
"What object should exist and how is it constructed?"

STRUCTURAL
"How should existing objects be connected/composed?"

BEHAVIORAL
"How should existing objects communicate and change behavior?"
```

And this connects directly to the SOLID discussion from the previous section:

```text
SOLID
  ↓
principles

DESIGN PATTERNS
  ↓
reusable structures that often help apply those principles

ARCHITECTURE
  ↓
larger-scale organization of those structures

FRAMEWORKS
  ↓
implementation infrastructure
```

So **patterns are not the architecture**. They are reusable solutions to recurring design problems inside an architecture.

The next useful step is to take all **23 patterns and build one realistic Java 17 application**—for example, the restaurant/order platform—and deliberately identify **where each pattern belongs, where two patterns overlap, and where using a pattern would actually make the code worse**. That is where the difference between *knowing the pattern* and *being able to design with patterns* becomes clear. I will try on next to update different application wise.
