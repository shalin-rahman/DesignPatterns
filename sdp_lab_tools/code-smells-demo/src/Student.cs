namespace CodeSmellsDemo;

public class Student
{
    // Primitive Obsession: every one of these is a real concept
    // (an identity, a contact address, an academic term, a grade) but
    // each is represented as a bare string or double instead of a type
    // that knows its own rules.
    public string Name = "";
    public string Id = "";
    public string Department = "";
    public string Email = "";
    public string Phone = "";
    public string Address = "";
    public string Semester = "";
    public string Cgpa = "";

    public double Mark1;
    public double Mark2;
    public double Mark3;

    // Duplicated Code: this averaging logic is copy-pasted in
    // Instructor.CalculateClassAverage with only the field names changed.
    public double CalculateResult()
    {
        double sum = Mark1 + Mark2 + Mark3;
        double average = sum / 3;

        if (average >= 80)
        {
            return 4.0;
        }
        else if (average >= 70)
        {
            return 3.5;
        }
        else if (average >= 60)
        {
            return 3.0;
        }
        else
        {
            return 2.0;
        }
    }
}
