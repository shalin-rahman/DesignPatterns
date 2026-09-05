class ShapeFactory
{
    Shape Create(int kind)
    {
        switch (kind)
        {
            case 1: return new Circle();
            case 2: return new Square();
            default: return new Triangle();
        }
    }
}

class Shape
{
}

class Circle : Shape
{
}

class Square : Shape
{
}

class Triangle : Shape
{
}
