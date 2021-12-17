#include "globals.h"

int s421()
{

//	storage classes and equivalencing
//	equivalence- no overlap

		yy = arr;
		for (int i = 0; i < LEN - 1; i++) {
			arr[i] = yy[i+1] + a[i];
		}
	int sum = 0;
	for (int i = 0; i < LEN; i++){
		sum += arr[i];
	}
  temp =sum;
	return 0;
}

