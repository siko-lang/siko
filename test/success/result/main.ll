%struct.Main_MyError = type { i32, [0 x i8] }

%struct.Main_MyError_Failure = type { i32, %struct.siko_Tuple_ }

%struct.Main_MySuccess = type { i32, [0 x i8] }

%struct.Main_MySuccess_Yay = type { i32, %struct.siko_Tuple_ }

%struct.Result_Result_Err_Main_MySuccess__Main_MyError = type { i32, %struct.siko_Tuple_Main_MyError }

%struct.Result_Result_Main_MySuccess__Main_MyError = type { i32, [4 x i8] }

%struct.Result_Result_Ok_Main_MySuccess__Main_MyError = type { i32, %struct.siko_Tuple_Main_MySuccess }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_MyError = type { %struct.Main_MyError }

%struct.siko_Tuple_Main_MySuccess = type { %struct.Main_MySuccess }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MyError, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Main_MySuccess(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MySuccess, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %tuple_12 = alloca %struct.siko_Tuple_, align 4
   %tuple_7 = alloca %struct.siko_Tuple_, align 4
   %unit_17 = alloca %struct.siko_Tuple_, align 4
   %matchValue_16 = alloca %struct.siko_Tuple_, align 4
   %match_var_4 = alloca %struct.siko_Tuple_, align 4
   %valueRef_3 = alloca %struct.Result_Result_Main_MySuccess__Main_MyError, align 4
   %a_2 = alloca %struct.Result_Result_Main_MySuccess__Main_MyError, align 4
   %call_1 = alloca %struct.Result_Result_Main_MySuccess__Main_MyError, align 4
   call void @Main_someFunc(ptr %call_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_2, ptr align 4 %call_1, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_3, ptr align 4 %a_2, i64 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_16, ptr align 4 %match_var_4, i64 0, i1 false)
   call void @siko_Tuple_(ptr %unit_17)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_17, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Result_Result_Main_MySuccess__Main_MyError, ptr %valueRef_3, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 0, label %block5
]

block3:
   %tmp_transform_5_1 = bitcast %struct.Result_Result_Main_MySuccess__Main_MyError* %valueRef_3 to %struct.Result_Result_Err_Main_MySuccess__Main_MyError*
   %transform_5 = getelementptr inbounds %struct.Result_Result_Err_Main_MySuccess__Main_MyError, ptr %tmp_transform_5_1, i32 0, i32 1
   %tupleField_6 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %transform_5, i32 0, i32 0
   br label %block4
block4:
   call void @siko_Tuple_(ptr %tuple_7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_4, ptr align 4 %tuple_7, i64 0, i1 false)
   br label %block1
block5:
   %tmp_transform_10_1 = bitcast %struct.Result_Result_Main_MySuccess__Main_MyError* %valueRef_3 to %struct.Result_Result_Ok_Main_MySuccess__Main_MyError*
   %transform_10 = getelementptr inbounds %struct.Result_Result_Ok_Main_MySuccess__Main_MyError, ptr %tmp_transform_10_1, i32 0, i32 1
   %tupleField_11 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %transform_10, i32 0, i32 0
   br label %block6
block6:
   call void @siko_Tuple_(ptr %tuple_12)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_4, ptr align 4 %tuple_12, i64 0, i1 false)
   br label %block1
}

define private void @Main_someFunc(ptr noundef %fn_result) {
block0:
   %call_2 = alloca %struct.Result_Result_Main_MySuccess__Main_MyError, align 4
   %call_1 = alloca %struct.Main_MyError, align 4
   call void @Main_MyError_Failure(ptr %call_1)
   call void @Result_Result_Err_Main_MySuccess__Main_MyError(ptr %call_1, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %call_2, i64 8, i1 false)
   ret void
}

define private void @Main_MyError_Failure(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_MyError_Failure, align 4
   %tag = getelementptr inbounds %struct.Main_MyError_Failure, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_MyError_Failure, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Result_Result_Err_Main_MySuccess__Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Result_Result_Err_Main_MySuccess__Main_MyError, align 4
   %tag = getelementptr inbounds %struct.Result_Result_Err_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Result_Result_Err_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


