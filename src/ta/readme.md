The "Hilbert Transform - Phasor Components" (HT_PHASOR) in TA-Lib computes the inphase and quadrature components of a time series, which are used to determine the dominant cycle phase and amplitude in the data. This function is based on John Ehlers' work on applying the Hilbert Transform in technical analysis.

Here's a step-by-step calculation for your price series $[10, 11, 12, \dots, 30]$, where $30$ is the latest value and $10$ is the oldest value.

**1. Initialize the Price Series**

First, list out your price series with their corresponding time indices:

| Time $t$ | Price $P(t)$ |
|------------|----------------|
| 0          | 10             |
| 1          | 11             |
| 2          | 12             |
| ...        | ...            |
| 20         | 30             |

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