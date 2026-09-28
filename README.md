The 5 GoF Creational Patterns — Java, Through One Restaurant

Let’s build the understanding from the ground up.

Imagine you’re opening a restaurant.

At first, everything is simple:

new Pizza()
new Chef()
new Oven()

But as the restaurant grows, creation gets complicated:

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

That’s where creational design patterns become useful.

They don’t mean:

“Never use new.”

They mean:

“Let’s control object creation when uncontrolled creation is making the design difficult to change, test, or maintain.”

⸻

The five patterns in one restaurant

Keep this mental picture throughout the tutorial:

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

The five patterns solve different creation problems.

⸻

1. Singleton

Restaurant story

You open the restaurant.

You decide:

“I need one kitchen manager.”

Why?

Imagine accidentally creating:

KitchenManager #1
KitchenManager #2
KitchenManager #3

Each manager thinks they control the kitchen.

Manager #1 says:

“Kitchen closes at 10.”

Manager #2 says:

“Kitchen closes at 11.”

Now you have inconsistent state.

The problem isn’t merely that there are multiple Java objects.

The problem is that the application expects one shared authority/state holder.

⸻

Without Singleton

KitchenManager manager1 = new KitchenManager();
KitchenManager manager2 = new KitchenManager();
System.out.println(manager1 == manager2);

Output:

false

These are two different objects.

⸻

Singleton idea

We want:

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

⸻

Basic implementation

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

Use it:

public class Main {
    public static void main(String[] args) {
        KitchenManager a = KitchenManager.getInstance();
        KitchenManager b = KitchenManager.getInstance();
        System.out.println(a == b);
    }
}

Output:

true

⸻

What’s actually happening?

1. private static KitchenManager instance

private static KitchenManager instance;

The class has one static variable capable of holding the instance.

static means it belongs to the class rather than each individual object.

⸻

2. Private constructor

private KitchenManager() {
}

This is extremely important.

Normally:

new KitchenManager();

would be possible.

But because the constructor is private:

new KitchenManager();

cannot be called from outside the class.

So the class controls its own creation.

⸻

3. getInstance()

public static KitchenManager getInstance()

This becomes the controlled entrance to the object.

First call:

instance == null
       ↓
create object
       ↓
store object
       ↓
return object

Later call:

instance != null
       ↓
return existing object

⸻

But there’s a concurrency problem

Suppose two threads arrive simultaneously:

Thread A                  Thread B
instance == null          instance == null
       │                         │
       ▼                         ▼
new Manager()             new Manager()

You could accidentally create two objects.

So the basic implementation isn’t sufficient for multithreaded applications.

A modern Java approach is:

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

This uses Java class initialization guarantees to safely initialize the instance.

⸻

But should you actually use Singleton?

Be careful.

Singleton creates global state.

For example:

KitchenManager.getInstance()

can be called from anywhere.

That creates a hidden dependency.

Testing also becomes harder because tests may share the same state.

That’s why modern applications often prefer dependency injection.

For example:

class RestaurantService {
    private final KitchenManager manager;
    RestaurantService(KitchenManager manager) {
        this.manager = manager;
    }
}

Now the dependency is obvious.

Spring can also manage an object as a singleton-scoped bean without you implementing the Singleton pattern yourself.

Remember

Singleton = control the application so there is one shared instance.

Not:

“Singleton is the best way to share objects.”

Those are very different statements.

⸻

2. Factory Method

Now the restaurant grows.

You offer:

Italian cuisine
Bengali cuisine

You need a chef.

But which chef?

ItalianChef
BengaliChef

The restaurant type should determine that.

⸻

Without Factory Method

You might write:

Chef chef;
if (type.equals("italian")) {
    chef = new ItalianChef();
} else if (type.equals("bengali")) {
    chef = new BengaliChef();
}

This works.

But imagine 20 restaurant types.

Now you have:

if (...)
else if (...)
else if (...)
else if (...)
else if (...)
...

And potentially the same logic appears in multiple places.

⸻

First create the product abstraction

interface Chef {
    void cook();
}

Concrete products:

class ItalianChef implements Chef {
    @Override
    public void cook() {
        System.out.println("Italian chef is cooking pasta.");
    }
}
class BengaliChef implements Chef {
    @Override
    public void cook() {
        System.out.println("Bengali chef is cooking biryani.");
    }
}

Now:

Chef
 ├── ItalianChef
 └── BengaliChef

⸻

Now the interesting part

Create an abstract restaurant:

abstract class Restaurant {
    public void prepareMeal() {
        System.out.println("Preparing restaurant meal...");
        Chef chef = createChef();
        chef.cook();
    }
    protected abstract Chef createChef();
}

Notice:

Chef chef = createChef();

The base class doesn’t know which concrete Chef it gets.

It just knows:

“I need a Chef.”

⸻

Italian restaurant

class ItalianRestaurant extends Restaurant {
    @Override
    protected Chef createChef() {
        return new ItalianChef();
    }
}

Bengali restaurant

class BengaliRestaurant extends Restaurant {
    @Override
    protected Chef createChef() {
        return new BengaliChef();
    }
}

Now:

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

Output:

Preparing restaurant meal...
Italian chef is cooking pasta.
Preparing restaurant meal...
Bengali chef is cooking biryani.

⸻

Why is this called Factory Method?

This method:

protected abstract Chef createChef();

is the Factory Method.

The base class says:

“I need a Chef.”

The subclass says:

“I’ll decide which Chef.”

Restaurant
    │
    └── createChef()
           │
       subclass decides
           │
      ┌────┴─────┐
      ▼          ▼
 ItalianChef  BengaliChef

The important point is inheritance.

The creator class defines the factory method, and subclasses override it.

⸻

Factory Method vs Simple Factory

This distinction matters.

A Simple Factory might look like:

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

That’s useful, but traditionally it’s called Simple Factory, not the GoF Factory Method pattern.

Factory Method looks more like:

abstract Creator
      │
      ├── common workflow
      │
      └── createProduct()
              ▲
              │
       overridden by subclass

Remember

Factory Method = subclasses decide which product gets created.

⸻

3. Abstract Factory

Now our restaurant becomes more complicated.

A chef alone isn’t enough.

We need:

Chef
Oven
Menu

And they must match the cuisine.

For Italian:

ItalianChef
ItalianOven
ItalianMenu

For Bengali:

BengaliChef
BengaliOven
BengaliMenu

This is a family.

That’s the key idea behind Abstract Factory.

⸻

Product interfaces

interface Chef {
    void cook();
}
interface Oven {
    void bake();
}
interface Menu {
    void show();
}

⸻

Italian family

class ItalianChef implements Chef {
    @Override
    public void cook() {
        System.out.println("Italian chef: cooking pasta.");
    }
}
class ItalianOven implements Oven {
    @Override
    public void bake() {
        System.out.println("Italian oven: baking pizza.");
    }
}
class ItalianMenu implements Menu {
    @Override
    public void show() {
        System.out.println("Italian menu.");
    }
}

⸻

Bengali family

class BengaliChef implements Chef {
    @Override
    public void cook() {
        System.out.println("Bengali chef: cooking biryani.");
    }
}
class BengaliOven implements Oven {
    @Override
    public void bake() {
        System.out.println("Bengali oven: baking naan.");
    }
}
class BengaliMenu implements Menu {
    @Override
    public void show() {
        System.out.println("Bengali menu.");
    }
}

⸻

Abstract Factory

interface RestaurantFactory {
    Chef createChef();
    Oven createOven();
    Menu createMenu();
}

This is the important interface.

It says:

“Every restaurant family must be able to provide these related products.”

⸻

Italian factory

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

Bengali:

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

⸻

Client

Now create the restaurant:

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

Notice what Restaurant does not know.

It doesn’t know:

ItalianChef
BengaliChef
ItalianOven
BengaliOven
ItalianMenu
BengaliMenu

It knows only:

Chef
Oven
Menu
RestaurantFactory

Run it:

public class Main {
    public static void main(String[] args) {
        RestaurantFactory factory =
                new ItalianRestaurantFactory();
        Restaurant restaurant =
                new Restaurant(factory);
        restaurant.open();
    }
}

Output:

Italian menu.
Italian chef: cooking pasta.
Italian oven: baking pizza.

Switch family:

RestaurantFactory factory =
        new BengaliRestaurantFactory();

The Restaurant code doesn’t change.

⸻

Why is Abstract Factory different from Factory Method?

This is the distinction I want you to remember.

Factory Method

You’re asking:

“Which Chef?”

Restaurant
     │
 createChef()
     │
 ┌───┴────┐
 ▼        ▼
Italian  Bengali
Chef     Chef

One product type.

⸻

Abstract Factory

You’re asking:

“Which entire family?”

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

Multiple related product types.

⸻

Where Abstract Factory appears in real software

UI themes

DarkFactory
 ├── DarkButton
 ├── DarkTextBox
 └── DarkCheckbox
LightFactory
 ├── LightButton
 ├── LightTextBox
 └── LightCheckbox

You don’t want a dark button with a light theme’s components accidentally.

⸻

Database family

PostgreSqlFactory
 ├── Connection
 ├── Command
 └── Transaction
SqlServerFactory
 ├── Connection
 ├── Command
 └── Transaction

⸻

Payment provider family

StripeFactory
 ├── Gateway
 ├── FraudChecker
 └── ReceiptGenerator
BkashFactory
 ├── Gateway
 ├── FraudChecker
 └── ReceiptGenerator

⸻

Cloud provider

AWSFactory
 ├── Storage
 ├── Queue
 └── Compute
AzureFactory
 ├── Storage
 ├── Queue
 └── Compute

The value is not merely “different implementations.”

It’s that the objects form a coherent family.

⸻

4. Builder

Back to the restaurant.

A customer wants:

Large pizza, thin crust, extra cheese, pepperoni, mushrooms, olives, no onions, spicy sauce…

A naive constructor becomes ugly:

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

What does this mean?

true, true, true, false

You have to remember which boolean means what.

This is the telescoping constructor problem: too many constructor parameters, especially optional ones.

⸻

Builder solution

We want:

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

Now the code explains itself.

⸻

Full implementation

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

Use:

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

⸻

What’s happening internally?

The Builder is essentially accumulating configuration:

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

The Pizza constructor is private:

private Pizza(Builder builder)

So users cannot casually construct an incomplete Pizza.

They go through:

build()

where validation can happen.

⸻

Why return this?

Consider:

public Builder size(String size) {
    this.size = size;
    return this;
}

It returns the same Builder.

Therefore:

builder
    .size(...)
    .crust(...)
    .cheese(...)

can be chained.

That’s called a fluent API.

⸻

Why is Pizza immutable?

We use:

private final String size;

and don’t provide setters.

Once built:

Pizza pizza = ...

its configuration cannot be changed.

That’s often desirable for value-like objects.

⸻

When NOT to use Builder

Don’t write:

new User.Builder()
    .name("John")
    .age(30)
    .build();

for an object that only has:

User(String name, int age)

This is probably simpler:

new User("John", 30);

Builder is useful when construction has enough complexity to justify it.

⸻

5. Prototype

Now imagine your restaurant has pizza templates.

You have a carefully configured:

Family Pizza
Large
Thin crust
Extra cheese
Pepperoni
Mushroom
Spicy sauce

You want 100 similar pizzas.

Why repeatedly reconstruct the configuration?

Instead:

“Take this existing pizza and make another one like it.”

That’s Prototype.

⸻

The problem with shallow copying

Suppose:

class Pizza {
    String name;
    List<String> toppings;
}

If you simply copy the object:

original
  │
  └── toppings ──────┐
                     ▼
                  List A
copy
  │
  └── toppings ──────┘

Both objects may point to the same mutable list.

Then:

copy.toppings.add("Olives");

could unexpectedly modify the original’s toppings too.

That’s a shallow-copy problem.

⸻

Deep copy

We want:

original
  │
  └── toppings → List A
copy
  │
  └── toppings → List B

The lists are separate.

⸻

Java Prototype example

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

Use:

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

Output:

Family Pizza -> [Cheese, Pepperoni, Mushrooms]
Family Pizza -> [Cheese, Pepperoni, Mushrooms, Olives]

The original wasn’t modified.

⸻

Why?

This constructor:

this.toppings = new ArrayList<>(toppings);

creates a new list.

So:

original.toppings → List A
copy.toppings     → List B

instead of:

original.toppings ─┐
                   ▼
                 List A
                   ▲
                   │
copy.toppings ─────┘

⸻

Is Java’s clone() always the answer?

No.

Java’s built-in Cloneable mechanism has some awkward historical behavior.

In modern Java, a method such as:

public Pizza copy()

can often be clearer and safer than exposing clone().

The Prototype pattern is the concept:

Create a new object by copying an existing object.

It doesn’t require you to use Object.clone().

⸻

When Prototype makes sense

Imagine a report system:

ReportTemplate
 ├── company logo
 ├── headers
 ├── formatting
 ├── sections
 └── configuration

Creating and configuring that from scratch every time could be expensive.

Instead:

ReportTemplate copy =
        template.copy();

Then customize it.

Good candidates include:

* document templates
* game objects
* preconfigured reports
* complex configuration objects
* expensive-to-create objects

But if:

new Pizza(...)

is cheap and simple, cloning it is probably unnecessary complexity.

⸻

Now let’s put the five side by side

Pattern	Problem	Main idea	Creates
Singleton	Need controlled single instance	One shared instance	One object
Factory Method	Subclasses need different products	Subclass decides creation	One product
Abstract Factory	Need compatible object families	Factory selects family	Multiple related products
Builder	Object has complicated construction	Build step-by-step	One complex object
Prototype	Creating similar objects is expensive/awkward	Copy existing object	New copy

⸻

The most important distinction

Imagine the restaurant owner asks five different questions.

Question 1

“How many kitchen managers should exist?”

Singleton

ONE

⸻

Question 2

“Which chef should this restaurant create?”

Factory Method

ItalianChef
OR
BengaliChef

⸻

Question 3

“Which complete cuisine family should this restaurant use?”

Abstract Factory

Italian:
Chef + Oven + Menu
OR
Bengali:
Chef + Oven + Menu

⸻

Question 4

“How should this complicated pizza be constructed?”

Builder

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

⸻

Question 5

“I already have a configured pizza. Can I make another one like it?”

Prototype

Existing Pizza
      │
    copy()
      │
      ▼
New Pizza

⸻

The if / switch question

You raised this earlier, and it is an important point.

Do these patterns eliminate:

if (...)
else if (...)
switch (...)

?

No.

The decision still has to happen somewhere.

Suppose configuration says:

restaurant.cuisine = ITALIAN

Somewhere the application has to turn that into:

ItalianRestaurantFactory

You could have:

if (cuisine == Cuisine.ITALIAN) {
    factory = new ItalianRestaurantFactory();
} else {
    factory = new BengaliRestaurantFactory();
}

That’s still an if.

The difference is where the decision happens.

You want:

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

rather than:

OrderService
    │
    ├── if Italian
    ├── if Bengali
    ├── create chef
    ├── create oven
    └── create menu

The decision doesn’t disappear.

It gets isolated.

You can also use:

Map<Cuisine, RestaurantFactory>

or dependency injection.

⸻

How to recognize the patterns in real code

When you encounter code, ask these questions.

“There should only be one.”

Think:

Singleton

But ask whether DI would be better.

⸻

“Something needs to decide which implementation to create.”

Think:

Factory

⸻

“A subclass determines which product gets created.”

Think:

Factory Method

⸻

“I need several related implementations that must belong together.”

Think:

Abstract Factory

⸻

“This object has 10 optional construction parameters.”

Think:

Builder

⸻

“I already have a configured object and want another similar object.”

Think:

Prototype

⸻

Now the SOLID connection

These patterns make more sense when you see the principles behind them.

⸻

SRP — Single Responsibility

A class should have a focused responsibility.

Bad:

Restaurant
 ├── business logic
 ├── create chef
 ├── create oven
 ├── create menu
 ├── configure database
 └── manage global state

Better:

Restaurant
     │
     └── business behavior
RestaurantFactory
     │
     └── object creation
Pizza.Builder
     │
     └── pizza construction

Singleton deserves caution here.

If your Singleton becomes:

KitchenManager
 ├── configuration
 ├── database
 ├── logging
 ├── payments
 ├── users
 └── everything else

you’ve created a God Object, not good design.

⸻

OCP — Open/Closed Principle

Open for extension, closed for modification.

Suppose tomorrow you add:

Japanese restaurant

With Abstract Factory:

JapaneseChef
JapaneseOven
JapaneseMenu
JapaneseRestaurantFactory

Existing client:

Restaurant restaurant =
        new Restaurant(factory);

doesn’t need to know the new concrete classes.

That’s a strong example of OCP.

⸻

LSP — Liskov Substitution

If:

Chef chef;

then:

ItalianChef

should behave as a valid Chef.

Likewise:

BengaliChef

should also be valid.

This would be suspicious:

class BengaliChef implements Chef {
    @Override
    public void cook() {
        throw new UnsupportedOperationException();
    }
}

If a class claims to be a Chef, but fundamentally can’t perform the operation promised by Chef, the abstraction is probably wrong.

⸻

ISP — Interface Segregation

Instead of:

interface RestaurantComponent {
    void cook();
    void bake();
    void showMenu();
    void processPayment();
    void cleanKitchen();
    void manageEmployees();
}

prefer focused interfaces:

interface Chef {
    void cook();
}
interface Oven {
    void bake();
}
interface Menu {
    void show();
}

Small interfaces are easier to implement and substitute.

⸻

DIP — Dependency Inversion

This:

class Restaurant {
    private ItalianChef chef =
            new ItalianChef();
}

couples Restaurant directly to ItalianChef.

Better:

class Restaurant {
    private final Chef chef;
    Restaurant(Chef chef) {
        this.chef = chef;
    }
}

Now:

Restaurant
     │
     ▼
   Chef
   ▲   ▲
   │   │
Italian Bengali

Restaurant depends on the abstraction.

⸻

Composition over inheritance

Factory Method commonly uses inheritance:

Restaurant
   ▲
   │
ItalianRestaurant

But Abstract Factory uses composition:

Restaurant restaurant =
        new Restaurant(factory);

The restaurant has a factory rather than becoming a subclass of a factory.

That can be easier to change at runtime.

For example:

Restaurant restaurant =
        new Restaurant(new ItalianRestaurantFactory());

or:

Restaurant restaurant =
        new Restaurant(new BengaliRestaurantFactory());

Same Restaurant class.

⸻

Encapsulate what varies

This is one of the most useful principles behind these patterns.

Ask:

What is likely to change?

In our restaurant:

Cuisine
Chef
Oven
Menu
Pizza configuration
Object copying

Then isolate those variations.

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

⸻

DRY

Factories can prevent repeated construction logic.

Builder can centralize validation.

But don’t misunderstand DRY.

This:

new User(name, age)

doesn’t need a factory simply because you wrote it twice.

Don’t create abstractions merely to eliminate a few repeated lines.

⸻

KISS

This is where developers often misuse design patterns.

If you have:

Pizza pizza = new Pizza("Large", "Thin");

don’t create:

PizzaFactory
PizzaBuilder
PizzaPrototypeRegistry
PizzaAbstractFactory
PizzaCreationStrategy

That’s ridiculous.

A pattern is justified by a design problem, not by the fact that a pattern exists.

⸻

YAGNI

“You aren’t gonna need it.”

If today your restaurant has:

one Chef
one Oven
one Menu

don’t build:

ItalianFactory
BengaliFactory
JapaneseFactory
ChineseFactory
IndianFactory

before you actually need them.

Design for known variation, not imaginary complexity.

⸻

Immutability

Builder works especially well with immutable objects.

For example:

public final class Pizza {
    private final String size;
    private final String crust;
}

Once:

Pizza pizza = builder.build();

the pizza doesn’t suddenly change because some other part of the application modified the Builder.

That gives you safer objects.

⸻

A practical decision tree

Here’s a slightly more accurate version of the decision process:

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

One correction to a simplistic decision tree is important:

“There are variants” does not automatically mean Factory Method.

You should first ask why the variation exists and where the creation decision belongs.

⸻

One mini-project using all five

Let’s put everything together.

Imagine our restaurant management application.

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

⸻

Singleton

KitchenManager manager =
        KitchenManager.getInstance();

Why?

One shared kitchen-management instance.

⸻

Factory Method

abstract class Restaurant {
    public void prepareMeal() {
        Chef chef = createChef();
        chef.cook();
    }
    protected abstract Chef createChef();
}

Italian restaurant:

class ItalianRestaurant extends Restaurant {
    @Override
    protected Chef createChef() {
        return new ItalianChef();
    }
}

Why?

The subclass determines the chef.

⸻

Abstract Factory

interface RestaurantFactory {
    Chef createChef();
    Oven createOven();
    Menu createMenu();
}

Italian factory:

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

Why?

We need an entire compatible cuisine family.

⸻

Builder

Pizza pizza =
        new Pizza.Builder()
                .size("large")
                .crust("thin")
                .cheese(true)
                .pepperoni(true)
                .build();

Why?

Pizza construction has many optional properties.

⸻

Prototype

Pizza secondPizza =
        originalPizza.copy();

Why?

We want another object based on an existing configuration.

⸻

Final mental model

Forget the formal definitions for a moment.

Imagine you’re standing in your restaurant kitchen.

Someone asks:

“Who manages this kitchen?”

Singleton

One kitchen manager.

⸻

Someone asks:

“Which chef should this restaurant create?”

Factory Method

The restaurant type decides.

⸻

Someone asks:

“Which complete kitchen setup are we using?”

Abstract Factory

Give me the whole Italian family or the whole Bengali family.

⸻

Someone asks:

“How do I construct this highly customized pizza?”

Builder

Configure it step by step and then build it.

⸻

Someone asks:

“I want another pizza exactly like this one, then I’ll customize it.”

Prototype

Copy the existing pizza.

⸻

The five words to memorize

Singleton       → ONE
Factory Method  → WHICH ONE?
Abstract Factory → WHICH FAMILY?
Builder         → HOW TO BUILD?
Prototype       → COPY

And the bigger lesson is even more important:

Design patterns are not goals. They’re tools for managing complexity.

If new Pizza() is perfectly clear, use new Pizza().

If a constructor has 15 optional parameters, consider Builder.

If subclasses need to choose a product, consider Factory Method.

If you need a compatible family of products, consider Abstract Factory.

If you need to duplicate configured objects, consider Prototype.

If you genuinely require one shared instance, consider Singleton—but in dependency-injected applications, let the DI container manage that lifecycle when possible.


# DesignPatterns and Scent Lab

This repository contains teaching material and working tools for studying
design patterns, code smells, static analysis, and LLM-integrating software.
The main executable projects are:

| Project | Purpose | Primary technology |
|---|---|---|
| [`sdp_lab_tools/scent`](sdp_lab_tools/scent/) | Analyze C# projects for structural code smells, metrics, design-principle risks, and quality-gate violations | Rust + Tree-sitter |
| [`sdp_lab_tools/scent-llm`](sdp_lab_tools/scent-llm/) | Generate code through configurable LLM providers and detect five LLM code smells in Python calls | Python |
| [`sdp_lab_tools/code-smells-demo`](sdp_lab_tools/code-smells-demo/) | Small intentionally smelly C# application for demonstrations and refactoring practice | C#/.NET |

## Choose a starting point

- Want to analyze a C# project? Start with
  [`scent/README.md`](sdp_lab_tools/scent/README.md).
- Want to learn the complete SCENT pipeline? Read the
  [`SCENT learning guide`](sdp_lab_tools/scent/docs/LEARNING_GUIDE.md).
- Want exact CLI commands and captured examples? Read the
  [`SCENT CLI workflow guide`](sdp_lab_tools/scent/docs/CLI_WORKFLOW_GUIDE.md).
- Want to generate code or analyze LLM API calls? Start with
  [`scent-llm/README.md`](sdp_lab_tools/scent-llm/README.md), then use the
  [full scent-llm guide](sdp_lab_tools/scent-llm/docs/GUIDE.md).
- Want a hands-on refactoring exercise? Open the
  [`code-smells-demo` refactoring guide](sdp_lab_tools/code-smells-demo/REFACTORING_GUIDE.md).

## Quick start: SCENT

SCENT parses C# source directly; it does not compile or execute the analyzed
project. From [`sdp_lab_tools/scent/`](sdp_lab_tools/scent/):

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

cargo run -p scent-cli -- analyze <path-to-csharp-project> --format table
cargo run -p scent-cli -- analyze <path-to-csharp-project> --format json
cargo run -p scent-cli -- gate <path-to-csharp-project> --max-critical 0 --max-high 0
```

For repeated use, install the CLI:

```powershell
cargo install --path crates/scent-cli
scent analyze <path-to-csharp-project> --format table
```

Configuration is read from the analyzed project when present:

- [`scent/smell_detector.toml`](sdp_lab_tools/scent/smell_detector.toml) shows
  project-level defaults.
- [`SCENT status`](sdp_lab_tools/scent/docs/status.md) distinguishes
  implemented behavior from planned work.
- [`JSON output reference`](sdp_lab_tools/scent/docs/JSON_OUTPUT_REFERENCE.md)
  documents report fields, locations, evidence, and fingerprints.

## Quick start: scent-llm

From [`sdp_lab_tools/scent-llm/`](sdp_lab_tools/scent-llm/):

```powershell
python -m pip install -e .

# No network required:
python -m scent_llm.cli analyze examples\smelly_example.py --json
python -m scent_llm.cli sandbox examples\smelly_example.py --language python --no-docker

# Local provider:
ollama pull llama3.1:8b
ollama serve
scent-llm generate "write a Python CSV validator" --out validator.py --analyze
```

Cloud providers are opt-in and use environment variables for secrets:

```powershell
$env:GROQ_API_KEY = "your-key"
scent-llm generate "write a bounded retry helper" --provider groq --analyze

$env:GEMINI_API_KEY = "your-key"
scent-llm generate "write a Python JSON validator" `
  --provider gemini --model gemini-3-flash-preview --analyze
```

Configuration precedence is:

```text
CLI flag > environment variable > scent_llm.toml in the current directory > built-in default
```

See the [scent-llm configuration and implementation guide](sdp_lab_tools/scent-llm/docs/GUIDE.md)
for provider setup, Gemini details, Hugging Face future integration, smell
definitions, sandbox isolation, troubleshooting, and tests.

## SpecDetect4LLM research tool

The separate SpecDetect4LLM research checkout has a Windows setup guide at
[`sdp_lab_tools/specDetect4LLM_setuo_guide.md`](sdp_lab_tools/specDetect4LLM_setuo_guide.md).
It covers Python 3.11 setup, the detector and web application, Docker usage,
tests, repository layout, prevalence analysis, and the `R25`–`R29`
LLM-integration smell rules. Use it to compare the research implementation
with the smaller [`scent-llm`](sdp_lab_tools/scent-llm/) reference tool.

## Learning materials

The [`sdp_lab_tools/materials/`](sdp_lab_tools/materials/) directory contains
the source material used by the tools and exercises:

- [`Code Smells lecture`](sdp_lab_tools/materials/Code_Smells_Lecture.pdf)
- [`Design Patterns lecture`](sdp_lab_tools/materials/lectutre_1_Design_Patterns.pdf)
- [`LLM Code Smells taxonomy and detection paper`](sdp_lab_tools/materials/LLM%20Code%20Smells-%20A%20Taxonomy%20and%20Detection%20Approach.pdf)
- [`Specification and Detection of LLM Code Smells paper`](sdp_lab_tools/materials/Specification%20and%20Detection%20of%20LLM%20Code%20Smells%20paper.pdf)
- [`LLM Code Smells and Design Patterns lecture`](sdp_lab_tools/materials/LLM_Code_Smells_Design_Patterns_Lecture_v2.pdf)

The LLM tool implements the original five-smell catalog. The newer taxonomy
paper expands that catalog; the difference and current implementation limits
are documented in the [scent-llm guide](sdp_lab_tools/scent-llm/docs/GUIDE.md).

## Repository map

```text
.
├── sdp_lab_tools/
│   ├── scent/          Rust C# static analyzer
│   ├── scent-llm/      Python LLM generator and smell analyzer
│   ├── code-smells-demo/
│   ├── materials/
│   └── docs/
└── README.md
```

Project-specific contribution instructions and verification commands live in:

- [`scent contributor guide`](sdp_lab_tools/scent/docs/contributor-guide.md)
- [`scent architecture`](sdp_lab_tools/scent/docs/architecture.md)
- [`scent-llm full guide`](sdp_lab_tools/scent-llm/docs/GUIDE.md)
