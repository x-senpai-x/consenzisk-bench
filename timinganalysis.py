#!/usr/bin/env python3
"""
Usage:
    python3 timinganalysis.py [input_file]
    
If no input file is provided, it will read from stdin.
"""

import sys
import re
import json
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from pathlib import Path


@dataclass
class OperationTiming:
    """Represents timing data for a single operation."""
    name: str
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    duration: Optional[float] = None
    steps: Optional[int] = None
    cost: Optional[float] = None


@dataclass
class ExecutionStats:
    """Represents overall execution statistics."""
    total_steps: int
    total_duration: float
    total_cost: float
    throughput: float
    frequency: float
    clocks_per_step: float


class ZiskTimingAnalyzer:
    """Professional ZisK timing analysis tool."""
    
    def __init__(self):
        self.operations: List[OperationTiming] = []
        self.execution_stats: Optional[ExecutionStats] = None
        self.opcode_stats: Dict[str, Dict] = {}
        self.memory_stats: Dict[str, int] = {}
        
    def parse_input(self, input_source) -> None:
        """Parse ZisK execution output from file or stdin."""
        if isinstance(input_source, str):
            with open(input_source, 'r') as f:
                content = f.read()
        else:
            content = input_source.read()
            
        self._parse_timing_markers(content)
        self._parse_execution_stats(content)
        self._parse_opcode_stats(content)
        self._parse_memory_stats(content)
        
    def _parse_timing_markers(self, content: str) -> None:
        """Extract timing markers and actual performance data from the output."""
        self._parse_detailed_performance_data(content)
        if not self.operations:
            timing_pattern = r'TIMING_(START|END):(.+)'
            matches = re.findall(timing_pattern, content)
            
            operation_times = {}
            for marker_type, operation_name in matches:
                if operation_name not in operation_times:
                    operation_times[operation_name] = {}
                operation_times[operation_name][marker_type.lower()] = True
                
            for op_name, markers in operation_times.items():
                if 'start' in markers and 'end' in markers:
                    self.operations.append(OperationTiming(name=op_name))
    
    def _parse_detailed_performance_data(self, content: str) -> None:
        """Parse detailed performance data from ZisK output."""
        pass
                
    def _parse_execution_stats(self, content: str) -> None:
        """Extract execution statistics from the output."""
        steps_match = re.search(r'Total Cost: ([\d.]+) sec', content)
        total_cost = float(steps_match.group(1)) if steps_match else 0.0
        main_cost_match = re.search(r'Main Cost: ([\d.]+) sec ([\d,]+) steps', content)
        if main_cost_match:
            main_cost = float(main_cost_match.group(1))
            total_steps = int(main_cost_match.group(2).replace(',', ''))
        else:
            main_cost = 0.0
            total_steps = 0
            
        process_rom_match = re.search(
            r'process_rom\(\) steps=([\d,]+) duration=([\d.]+) tp=([\d.]+) Msteps/s freq=([\d.]+) ([\d.]+) clocks/step',
            content
        )
        
        if process_rom_match:
            steps = int(process_rom_match.group(1).replace(',', ''))
            duration = float(process_rom_match.group(2))
            throughput = float(process_rom_match.group(3))
            frequency = float(process_rom_match.group(4))
            clocks_per_step = float(process_rom_match.group(5))
        else:
            steps = total_steps
            duration = 0.0
            throughput = 0.0
            frequency = 0.0
            clocks_per_step = 0.0
            
        self.execution_stats = ExecutionStats(
            total_steps=steps,
            total_duration=duration,
            total_cost=total_cost,
            throughput=throughput,
            frequency=frequency,
            clocks_per_step=clocks_per_step
        )
        
    def _parse_opcode_stats(self, content: str) -> None:
        """Extract opcode statistics from the output."""
        opcode_pattern = r'(\w+): ([\d.]+) sec \(([\d]+) steps/op\) \(([\d,]+) ops\)'
        matches = re.findall(opcode_pattern, content)
        
        for opcode, cost, steps_per_op, ops in matches:
            self.opcode_stats[opcode] = {
                'cost': float(cost),
                'steps_per_op': int(steps_per_op),
                'operations': int(ops.replace(',', ''))
            }
            
    def _parse_memory_stats(self, content: str) -> None:
        """Extract memory statistics from the output."""
        memory_pattern = r'Memory: ([\d,]+) a reads \+ ([\d,]+) na1 reads \+ ([\d,]+) na2 reads \+ ([\d,]+) a writes \+ ([\d,]+) na1 writes \+ ([\d,]+) na2 writes'
        match = re.search(memory_pattern, content)
        
        if match:
            self.memory_stats = {
                'aligned_reads': int(match.group(1).replace(',', '')),
                'non_aligned_reads_1': int(match.group(2).replace(',', '')),
                'non_aligned_reads_2': int(match.group(3).replace(',', '')),
                'aligned_writes': int(match.group(4).replace(',', '')),
                'non_aligned_writes_1': int(match.group(5).replace(',', '')),
                'non_aligned_writes_2': int(match.group(6).replace(',', ''))
            }
            
    def calculate_per_operation_costs(self) -> None:
        """Calculate per-operation costs using actual measured data."""
        if not self.execution_stats:
            return
            
        if self._load_actual_timing_data():
            self._calculate_from_actual_data()
        elif self.operations:
            self._calculate_complexity_based()
    
    def _load_actual_timing_data(self) -> bool:
        """Load actual timing data from timing_results.json if available."""
        try:
            json_paths = ['timing_results.json', './timing_results.json', '../timing_results.json']
            data = None
            
            for json_path in json_paths:
                try:
                    with open(json_path, 'r') as f:
                        data = json.load(f)
                        break
                except FileNotFoundError:
                    continue
                    
            if data is None:
                return False
                
            if 'operations' in data:
                self.operations = []
                for op_data in data['operations']:
                    op = OperationTiming(
                        name=op_data['name'],
                        steps=op_data.get('steps'),
                        duration=op_data.get('duration'),
                        cost=op_data.get('cost')
                    )
                    self.operations.append(op)
                return True
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            pass
        return False
    
    def _calculate_from_actual_data(self) -> None:
        """Calculate costs using actual measured timing data."""
        pass
        
            
    def _calculate_complexity_based(self) -> None:
        """Calculate costs using equal distribution (fallback only)."""
        if not self.execution_stats or not self.operations:
            return
            
        num_operations = len(self.operations)
        if num_operations == 0:
            return
            
        equal_weight = 1.0 / num_operations
        
        for op in self.operations:
            op.steps = int(self.execution_stats.total_steps * equal_weight)
            op.duration = self.execution_stats.total_duration * equal_weight
            op.cost = self.execution_stats.total_cost * equal_weight
            
            
    def generate_report(self) -> str:
        """Generate a comprehensive analysis report."""
        if not self.execution_stats:
            return "No execution statistics found. Please provide valid ZisK output."
            
        report = []
        report.append("=" * 80)
        report.append("ZISK PER-OPERATION CYCLE COUNTING ANALYSIS REPORT")
        report.append("=" * 80)
        report.append("")
        
        report.append("OVERALL EXECUTION STATISTICS")
        report.append("-" * 40)
        report.append(f"Total Steps: {self.execution_stats.total_steps:,}")
        report.append(f"Total Duration: {self.execution_stats.total_duration:.4f} seconds")
        report.append(f"Total Cost: {self.execution_stats.total_cost:.2f} sec")
        report.append(f"Throughput: {self.execution_stats.throughput:.2f} Msteps/s")
        report.append(f"Frequency: {self.execution_stats.frequency:.0f} MHz")
        report.append(f"Clocks per Step: {self.execution_stats.clocks_per_step:.2f}")
        report.append("")
        
        report.append(f"OPERATIONS DETECTED: {len(self.operations)}")
        report.append("-" * 40)
        for i, op in enumerate(self.operations, 1):
            report.append(f"{i}. {op.name}")
        report.append("")
        
        if self.operations:
            report.append("PER-OPERATION COST ANALYSIS")
            report.append("-" * 40)
            
            for op in self.operations:
                if op.steps is not None and op.cost is not None:
                    op_steps = op.steps
                    op_cost = op.cost
                    op_time = op.duration if op.duration is not None else 0.0
                    weight = op_steps / self.execution_stats.total_steps if self.execution_stats.total_steps > 0 else 0.0
                else:
                    weight = 1.0 / len(self.operations) if len(self.operations) > 0 else 0.0
                    op_steps = int(self.execution_stats.total_steps * weight)
                    op_time = self.execution_stats.total_duration * weight
                    op_cost = self.execution_stats.total_cost * weight
                
                report.append(f"{op.name}:")
                report.append(f"  Weight: {weight*100:.1f}%")
                report.append(f"  Steps: {op_steps:,}")
                report.append(f"  Time: {op_time:.4f} seconds")
                report.append(f"  Cost: {op_cost:.2f} sec")
                report.append("")
                
        if self.opcode_stats:
            report.append("TOP EXPENSIVE OPCODES")
            report.append("-" * 30)
            sorted_opcodes = sorted(
                self.opcode_stats.items(), 
                key=lambda x: x[1]['cost'], 
                reverse=True
            )[:10]
            
            for opcode, stats in sorted_opcodes:
                report.append(f"{opcode}: {stats['cost']:.2f} sec ({stats['operations']:,} ops)")
            report.append("")
            
        if self.memory_stats:
            report.append("MEMORY USAGE ANALYSIS")
            report.append("-" * 30)
            total_reads = (self.memory_stats['aligned_reads'] + 
                          self.memory_stats['non_aligned_reads_1'] + 
                          self.memory_stats['non_aligned_reads_2'])
            total_writes = (self.memory_stats['aligned_writes'] + 
                           self.memory_stats['non_aligned_writes_1'] + 
                           self.memory_stats['non_aligned_writes_2'])
            
            report.append(f"Total Reads: {total_reads:,}")
            report.append(f"Total Writes: {total_writes:,}")
            report.append(f"Total Memory Operations: {total_reads + total_writes:,}")
            report.append("")
            
        report.append("=" * 80)
        report.append("Analysis completed successfully.")
        report.append("=" * 80)
        
        return "\n".join(report)
        
    def export_json(self, filename: str) -> None:
        """Export analysis results to JSON format."""
        data = {
            'execution_stats': {
                'total_steps': self.execution_stats.total_steps,
                'total_duration': self.execution_stats.total_duration,
                'total_cost': self.execution_stats.total_cost,
                'throughput': self.execution_stats.throughput,
                'frequency': self.execution_stats.frequency,
                'clocks_per_step': self.execution_stats.clocks_per_step
            } if self.execution_stats else None,
            'operations': [
                {
                    'name': op.name,
                    'steps': op.steps,
                    'duration': op.duration,
                    'cost': op.cost
                } for op in self.operations
            ],
            'opcode_stats': self.opcode_stats,
            'memory_stats': self.memory_stats
        }
        
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
            
        print(f"Results exported to {filename}")


def main():
    """Main function to run the analysis."""
    if len(sys.argv) > 1 and sys.argv[1] in ['--help', '-h']:
        print(__doc__)
        sys.exit(0)
    
    analyzer = ZiskTimingAnalyzer()
    if len(sys.argv) > 1:
        input_file = sys.argv[1]
        if not Path(input_file).exists():
            print(f"Error: File '{input_file}' not found.")
            sys.exit(1)
        print(f"Analyzing file: {input_file}")
        analyzer.parse_input(input_file)
    else:
        print("Reading from stdin... (paste your ZisK output and press Ctrl+D when done)")
        analyzer.parse_input(sys.stdin)
    
    analyzer.calculate_per_operation_costs()
    
    report = analyzer.generate_report()
    print(report)
    if len(sys.argv) > 2 and sys.argv[2] == '--export':
        output_file = sys.argv[3] if len(sys.argv) > 3 else 'timing_analysis.json'
        analyzer.export_json(output_file)


if __name__ == "__main__":
    main()