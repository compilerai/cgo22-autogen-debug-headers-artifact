#include "globals.h"

TYPE sum1d(){
	TYPE ret = 0.;
	for (int i = 0; i < LEN; i++)
		ret += arr[i];
	return ret;
}


