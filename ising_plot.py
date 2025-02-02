import matplotlib.pyplot as plt
import numpy as np

def read_data(filename):
    data = []
    with open(filename, 'r') as file:
        for line in file:
            temp, mag = map(float, line.strip().split(','))
            data.append((temp, mag))
    return data

def process_data(data, discard_bottom=True):
    temp_dict = {}
    for temp, mag in data:
        if temp not in temp_dict:
            temp_dict[temp] = []
        temp_dict[temp].append(mag)
    
    processed_data = []
    for temp, mags in temp_dict.items():
        mags.sort()
        if discard_bottom:
            subset = mags[len(mags)//2:]
        else:
            subset = mags[:]
        mean = np.median(subset + [1])
        min_val = np.min(subset)
        max_val = np.max(subset)
        processed_data.append((temp, mean, min_val, max_val))
    
    return processed_data

# I wrote this with copilot as I usually do when writing mpl code
def plot_data(processed_data, output_path):
    plt.clf()  # comment this line for stacked figures
    temps, means, min_vals, max_vals = zip(*processed_data)
    print(f"Building {output_path}")
    syst_width = int(output_path.split("/")[-1].split("_")[0])
    yerr = [means - np.array(min_vals), np.array(max_vals) - means]
    plt.errorbar(temps, means, yerr=yerr, fmt='o', capsize=5, label=f'{syst_width} Magnetization')
    plt.xlabel('Temperature')
    plt.ylabel('Magnetization')
    plt.axvline(x=2.27, color='gray', linestyle='--', label='Critical Temp 2.27')
    plt.ylim(0, 1)
    plt.title(f'Mean Absolute Magnetization vs Temperature')
    plt.legend()
    plt.savefig(output_path)
    # plt.show()

def handle(input_file):
    output_file = input_file[:-4] + "_python.png"  # Change this to your desired output file
    data = read_data(input_file)
    processed_data = process_data(data, discard_bottom=False)
    plot_data(processed_data, output_file)


import glob
import os
if __name__ == "__main__":
    txt_files = glob.glob(os.path.join(os.getcwd(), "*.txt"))
    for input_file in txt_files:
        handle(input_file)