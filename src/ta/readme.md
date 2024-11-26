# Formula

- [Formula](#formula)
  - [Hilbert Transform](#hilbert-transform)
  - [Confidence Bands and Prediction Bands](#confidence-bands-and-prediction-bands)

## Hilbert Transform

The "Hilbert Transform - Phasor Components" (HT_PHASOR) in TA-Lib computes the inphase and quadrature components of a time series, which are used to determine the dominant cycle phase and amplitude in the data. This function is based on John Ehlers' work on applying the Hilbert Transform in technical analysis.

Here's a step-by-step calculation for your price series $[10, 11, 12, \dots, 30]$, where $30$ is the latest value and $10$ is the oldest value.

**1. Initialize the Price Series**

First, list out your price series with their corresponding time indices:

| Time $t$ | Price $P(t)$ |
| -------- | ------------ |
| 0        | 10           |
| 1        | 11           |
| 2        | 12           |
| ...      | ...          |
| 20       | 30           |

**2. Calculate the Smoothed Price**

The smoothed price is calculated using a weighted moving average:

$$
\text{SmoothPrice}(t) = \frac{4 \times P(t) + 3 \times P(t-1) + 2 \times P(t-2) + P(t-3)}{10}
$$

*Note:* This calculation starts from $t = 3$ since we need $P(t-3)$.

**Example Calculation for $t = 3$:**

$$
\begin{align*}
\text{SmoothPrice}(3) &= \frac{4 \times P(3) + 3 \times P(2) + 2 \times P(1) + P(0)}{10} \\
&= \frac{4 \times 13 + 3 \times 12 + 2 \times 11 + 10}{10} \\
&= \frac{52 + 36 + 22 + 10}{10} = \frac{120}{10} = 12
\end{align*}
$$

Repeat this calculation for all $t$ from $3$ to $20$.

**3. Compute the Detrender**

The detrender is calculated using a discrete approximation of the Hilbert Transform:

$$
\text{Detrender}(t) = 0.0962 \times \text{SmoothPrice}(t) + 0.5769 \times \text{SmoothPrice}(t-2) - 0.5769 \times \text{SmoothPrice}(t-4) - 0.0962 \times \text{SmoothPrice}(t-6)
$$

*Note:* This calculation starts from $t = 6$ since we need $\text{SmoothPrice}(t-6)$.

**Example Calculation for $t = 6$:**

First, calculate the necessary smoothed prices:

- $\text{SmoothPrice}(6)$ (already calculated)
- $\text{SmoothPrice}(4)$
- $\text{SmoothPrice}(2)$
- $\text{SmoothPrice}(0)$

Assuming you have:

- $\text{SmoothPrice}(6) = 15$
- $\text{SmoothPrice}(4) = 13$
- $\text{SmoothPrice}(2) = 11$
- $\text{SmoothPrice}(0) = 10$

Now, compute the detrender:

$$
\begin{align*}
\text{Detrender}(6) &= 0.0962 \times 15 + 0.5769 \times 13 - 0.5769 \times 11 - 0.0962 \times 10 \\
&= 1.443 + 7.4997 - 6.3459 - 0.962 \\
&= 1.6348
\end{align*}
$$

Repeat this calculation for all $t$ from $6$ to $20$.

**4. Calculate the Inphase and Quadrature Components**

- **Inphase Component:**

  $$
  \text{InPhase}(t) = \text{Detrender}(t - 3)
  $$

- **Quadrature Component:**

  $$
  \text{Quadrature}(t) = 0.0962 \times \text{Detrender}(t) + 0.5769 \times \text{Detrender}(t-2) - 0.5769 \times \text{Detrender}(t-4) - 0.0962 \times \text{Detrender}(t-6)
  $$

*Note:* The quadrature component calculation starts from $t = 12$ since it requires $\text{Detrender}(t-6)$.

**Example Calculation for $t = 12$:**

Assuming you have the necessary detrender values:

- $\text{Detrender}(12)$
- $\text{Detrender}(10)$
- $\text{Detrender}(8)$
- $\text{Detrender}(6)$

Compute the quadrature component:

$$
\begin{align*}
\text{Quadrature}(12) &= 0.0962 \times \text{Detrender}(12) + 0.5769 \times \text{Detrender}(10) - 0.5769 \times \text{Detrender}(8) - 0.0962 \times \text{Detrender}(6) \\
&= \text{(Insert computed values)}
\end{align*}
$$

Similarly, compute the inphase component:

$$
\text{InPhase}(12) = \text{Detrender}(9)
$$

**5. Compute the Phase Angle (Optional)**

The phase angle helps in identifying the cycle:

$$
\text{Phase}(t) = \arctan\left(\frac{\text{Quadrature}(t)}{\text{InPhase}(t)}\right) \times \left(\frac{180}{\pi}\right)
$$

**6. Repeat for All Time Steps**

Perform these calculations for each time step $t$ where the data is sufficient (considering the lags in calculations).

**Summary**

- **SmoothPrice**: Starts from $t = 3$.
- **Detrender**: Starts from $t = 6$.
- **InPhase and Quadrature**: Start from $t = 12$.

**Note on Practical Implementation**

Due to the initial data requirements (lags), the inphase and quadrature components can only be calculated after a certain number of data points. In practice, software libraries like TA-Lib handle these initial periods internally.

**Final Output**

The inphase and quadrature arrays will contain the computed values starting from their respective starting points, with `NaN` or zeros for the initial periods where the calculation couldn't be performed.

---

**Please Note:** The above calculations are illustrative. In a real-world scenario, you would perform these calculations using precise floating-point arithmetic, ensuring that all intermediate steps are accurately computed. Additionally, edge cases and initial conditions need to be handled carefully.

## Confidence Bands and Prediction Bands

To calculate the 95% confidence bands and 95% prediction bands for the regression line obtained through Least Squares, follow these steps:

1. **Least Squares Regression**  
Given a set of data points $(x_i, y_i)$ for $i = 1, 2, ..., n$, the linear regression model is typically written as:
$$
y = \beta_0 + \beta_1 x + \epsilon
$$
where $\beta_0$ is the intercept, $\beta_1$ is the slope, and $\epsilon$ is the error term.

You can obtain the Least Squares estimates $\hat{\beta}_0$ and $\hat{\beta}_1$ using the normal equations:
$$
\hat{\beta}_1 = \frac{n \sum_{i=1}^{n} x_i y_i - \sum_{i=1}^{n} x_i \sum_{i=1}^{n} y_i}{n \sum_{i=1}^{n} x_i^2 - (\sum_{i=1}^{n} x_i)^2}
$$
$$
\hat{\beta}_0 = \frac{1}{n} \sum_{i=1}^{n} y_i - \hat{\beta}_1 \frac{1}{n} \sum_{i=1}^{n} x_i
$$
These provide the best-fit line.

2. **Residuals and Standard Error of the Estimate**  
The residuals are the differences between the observed values and the predicted values:
$$
e_i = y_i - (\hat{\beta}_0 + \hat{\beta}_1 x_i)
$$
The residual sum of squares (RSS) is:
$$
RSS = \sum_{i=1}^{n} e_i^2
$$
The standard error of the estimate is:
$$
SE_{\text{est}} = \sqrt{\frac{RSS}{n - 2}}
$$
where $n - 2$ is the degrees of freedom (since two parameters, $\hat{\beta}_0$ and $\hat{\beta}_1$, are estimated).

3. **95% Confidence Bands**  
The confidence band represents the uncertainty around the regression line at a given point $x$. The formula for the 95% confidence interval around the estimated mean response at a particular $x_0$ is:
$$
\hat{y}_0 \pm t_{\alpha/2, n-2} \cdot SE_{\hat{y}_0}
$$
where:
- $\hat{y}_0 = \hat{\beta}_0 + \hat{\beta}_1 x_0$ is the predicted value at $x_0$,
- $SE_{\hat{y}_0} = SE_{\text{est}} \sqrt{\frac{1}{n} + \frac{(x_0 - \bar{x})^2}{\sum_{i=1}^{n} (x_i - \bar{x})^2}}$ is the standard error of the predicted value at $x_0$,
- $t_{\alpha/2, n-2}$ is the critical value from the $t$-distribution with $n-2$ degrees of freedom and $\alpha = 0.05$ (for a 95% confidence level).

The confidence bands account for the variability in estimating the regression line.

4. **95% Prediction Bands**  
The prediction band is wider than the confidence band because it accounts not only for the uncertainty in estimating the regression line but also for the variability in the data points around the line. The formula for the 95% prediction interval at $x_0$ is:
$$
\hat{y}_0 \pm t_{\alpha/2, n-2} \cdot SE_{\text{pred}}
$$
where:
$$
SE_{\text{pred}} = SE_{\text{est}} \sqrt{1 + \frac{1}{n} + \frac{(x_0 - \bar{x})^2}{\sum_{i=1}^{n} (x_i - \bar{x})^2}}
$$
This interval is wider because it includes both the model error and the inherent variability in the data.

Summary of Key Formulas
- **Confidence interval around the regression line at $x_0$:**
  $$
  \hat{y}_0 \pm t_{\alpha/2, n-2} \cdot SE_{\hat{y}_0}
  $$
- **Prediction interval for a new observation at $x_0$:**
  $$
  \hat{y}_0 \pm t_{\alpha/2, n-2} \cdot SE_{\text{pred}}
  $$

Both bands require calculating the standard errors of the predicted value and the estimate based on residuals. The difference lies in how they account for the variability—confidence bands focus on the line itself, while prediction bands include both the line and future data points.

You can use these intervals to visually assess the precision of your regression model and the likely range of future observations at specific values of $x$.