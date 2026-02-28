use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ScoreGaugeProps {
    pub passed: u32,
    pub total: u32,
}

#[function_component(ScoreGauge)]
pub fn score_gauge(props: &ScoreGaugeProps) -> Html {
    let percentage = if props.total > 0 {
        ((props.passed as f64 / props.total as f64) * 100.0).round() as u32
    } else {
        0
    };

    let color = if percentage >= 90 {
        "#0cce6b"
    } else if percentage >= 50 {
        "#ffa400"
    } else {
        "#ff4e42"
    };

    let label = if percentage >= 90 {
        "Excellent"
    } else if percentage >= 70 {
        "Bon"
    } else if percentage >= 50 {
        "À améliorer"
    } else {
        "Insuffisant"
    };

    // SVG circular gauge (like PageSpeed Insights)
    let circumference = 2.0 * std::f64::consts::PI * 54.0;
    let dash_offset = circumference * (1.0 - percentage as f64 / 100.0);

    html! {
        <div class="score-gauge">
            <svg class="gauge-svg" viewBox="0 0 120 120" width="200" height="200">
                // Background circle
                <circle
                    cx="60" cy="60" r="54"
                    fill="none"
                    stroke="#e0e0e0"
                    stroke-width="8"
                />
                // Score arc
                <circle
                    cx="60" cy="60" r="54"
                    fill="none"
                    stroke={color.to_string()}
                    stroke-width="8"
                    stroke-linecap="round"
                    stroke-dasharray={format!("{}", circumference)}
                    stroke-dashoffset={format!("{}", dash_offset)}
                    transform="rotate(-90 60 60)"
                    class="gauge-arc"
                />
                // Score text
                <text
                    x="60" y="55"
                    text-anchor="middle"
                    class="gauge-score-text"
                    fill={color.to_string()}
                >
                    {percentage}
                </text>
                <text
                    x="60" y="75"
                    text-anchor="middle"
                    class="gauge-label-text"
                    fill="#5f6368"
                >
                    {format!("{}/{} checks", props.passed, props.total)}
                </text>
            </svg>
            <div class="gauge-badge" style={format!("color: {}", color)}>
                {label}
            </div>
        </div>
    }
}
