package abstract_factory_2_cars;

/* Abstract Factory Interface */
interface CarFactory {
    Car createCar();
    CarSpecification createSpecification();
}