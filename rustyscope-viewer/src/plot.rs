use plotly::common::Mode;
use plotly::{Plot, Scatter, ImageFormat};

pub fn line_y(y: impl IntoIterator<Item=f32> + ExactSizeIterator) {
    let x = (0..y.len()).into_iter().map(|i| i as f32);
    line(x,y);
}

pub fn two_lines(x: impl IntoIterator<Item=f32>, 
    y1: impl IntoIterator<Item=f32>, 
    y2: impl IntoIterator<Item=f32>) {

    let x1: Vec<f32> = x.into_iter().collect();
    let x2: Vec<f32> = x1.clone();
    let trace1 = Scatter::new(x1, y1)
        .name("trace1")
        .mode(Mode::Lines);
    let trace2 = Scatter::new(x2, y2)
        .name("trace2")
        .mode(Mode::Lines);

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace2);

    // The following will save the plot in all available formats and show the plot.
    // plot.save("scatter", ImageFormat::PNG,  1024, 680, 1.0);
    plot.show();
}

pub fn line(x: impl IntoIterator<Item=f32>, y: impl IntoIterator<Item=f32>) {
    let trace = Scatter::new(x, y)
        .name("trace2")
        .mode(Mode::Lines);

    let mut plot = Plot::new();
    plot.add_trace(trace);

    // The following will save the plot in all available formats and show the plot.
    // plot.save("scatter", ImageFormat::PNG,  1024, 680, 1.0);
    plot.show();
}
