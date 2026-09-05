namespace CodeSmellsDemo;

public class Instructor
{
    public string Name = "";

    public double SampleScore1;
    public double SampleScore2;
    public double SampleScore3;

    // Duplicated Code: same shape as Student.CalculateResult — sum three
    // scores, divide by three, map to a grade band — just renamed.
    public double CalculateClassAverage()
    {
        double sum = SampleScore1 + SampleScore2 + SampleScore3;
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
