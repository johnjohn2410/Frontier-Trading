import React from "react";
import Suggestions from "../components/Suggestions";
import Positions from "../components/Positions";

export default function Dashboard() {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto p-4">
        <header className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Frontier Trading</h1>
          <p className="text-gray-600 mt-2">AI-Powered Trading Platform</p>
        </header>
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <section className="lg:col-span-2">
            <Suggestions />
          </section>
          <section>
            <Positions />
          </section>
        </div>
        
        <footer className="mt-12 text-center text-gray-500 text-sm">
          <p>Frontier Trading Platform - Real-time AI Copilot</p>
          <p className="mt-1">Built with Rust, React, and Redis Streams</p>
        </footer>
      </div>
    </div>
  );
}
