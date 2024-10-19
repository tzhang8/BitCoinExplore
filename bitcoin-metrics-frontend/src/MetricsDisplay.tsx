import React, { useEffect, useState } from 'react';
import axios from 'axios';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js';
import { Line } from 'react-chartjs-2';

// Register chart components for BTC price graph
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

interface Metrics {
  block_height: number;
  btc_price: number;
  timestamp: string;
}

const MetricsDisplay: React.FC = () => {
  const [metrics, setMetrics] = useState<Metrics[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      const response = await axios.get<Metrics[]>('http://localhost:8080/api/metrics');

      setMetrics((prevMetrics) => {
        const combinedMetrics = [...prevMetrics, ...response.data];  
        const uniqueMetrics = combinedMetrics.filter(
          (metric, index, self) =>
            index === self.findIndex((m) => m.timestamp === metric.timestamp)  // Remove duplicates
        );
        return uniqueMetrics.slice(-10);  
      });
    } catch (err) {
      setError('Error fetching data');
    }
  };

  useEffect(() => {
    fetchMetrics();
    const intervalId = setInterval(fetchMetrics, 10000);
    return () => clearInterval(intervalId);
  }, []);

  const timestamps = metrics.map((m) => new Date(m.timestamp).toLocaleString());
  const btcPrices = metrics.map((m) => m.btc_price);  // Use raw btc_price values for more precision

  const priceChartData = {
    labels: timestamps,
    datasets: [
      {
        label: 'BTC Price (USD)',
        data: btcPrices,
        borderColor: '#00c9ff',
        backgroundColor: 'rgba(0, 201, 255, 0.2)',
        fill: true,
      },
    ],
  };

  const chartOptions = {
    responsive: true,
    plugins: {
      legend: {
        position: 'top' as const,
        labels: {
          color: '#ffffff', // White text for labels
        },
      },
      title: {
        display: true,
        text: 'Bitcoin Metrics',
        color: '#ffffff', // White text for title
      },
    },
    scales: {
      x: {
        ticks: {
          color: '#ffffff', // White text for x-axis labels
        },
        grid: {
          color: '#555', // Dark grid lines
        },
      },
      y: {
        ticks: {
          color: '#ffffff', // White text for y-axis labels
          callback: function (tickValue: string | number) {
            return tickValue;  
          },
        },
        grid: {
          color: '#555', // Dark grid lines
        },
      },
    },
  };

  return (
    <div style={{ backgroundColor: '#1c1c1c', minHeight: '100vh', padding: '20px', color: '#ffffff' }}>
      <h1>Bitcoin Metrics</h1>
      {error && <p style={{ color: 'red' }}>{error}</p>}

      {metrics.length > 0 ? (
        <div>
          <h2>BTC Price Chart</h2>
          <Line data={priceChartData} options={chartOptions} />

          <h2 style={{ marginTop: '40px' }}>Block Height Table</h2>
          <table style={{
            width: '100%',
            borderCollapse: 'collapse',
            marginTop: '10px',
            backgroundColor: '#333',
            color: '#ffffff',
            border: '1px solid #555',
          }}>
            <thead>
              <tr style={{ backgroundColor: '#444' }}>
                <th style={{ padding: '10px', border: '1px solid #555' }}>Time</th>
                <th style={{ padding: '10px', border: '1px solid #555' }}>Block Height</th>
                <th style={{ padding: '10px', border: '1px solid #555' }}>BTC Price (USD)</th>
              </tr>
            </thead>
            <tbody>
              {metrics.map((metric, idx) => (
                <tr key={idx} style={{ backgroundColor: idx % 2 === 0 ? '#3b3b3b' : '#2b2b2b' }}>
                  <td style={{ padding: '10px', border: '1px solid #555' }}>{new Date(metric.timestamp).toLocaleString()}</td>
                  <td style={{ padding: '10px', border: '1px solid #555' }}>{metric.block_height}</td>
                  <td style={{ padding: '10px', border: '1px solid #555' }}>{metric.btc_price}</td> {/* Show price with 8 decimals */}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p>Loading...</p>
      )}
    </div>
  );
};

export default MetricsDisplay;
