import { useEffect, useState } from "react";

export function useStream<T = any>(url: string) {
  const [items, setItems] = useState<T[]>([]);
  
  useEffect(() => {
    const ws = new WebSocket(url);
    
    ws.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        setItems(prev => [data, ...prev].slice(0, 200));
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
    
    ws.onclose = () => {
      console.log('WebSocket connection closed');
    };
    
    return () => ws.close();
  }, [url]);
  
  return items;
}
