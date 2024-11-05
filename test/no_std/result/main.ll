%struct.Main_MyError = type { i32, [0 x i8] }

%struct.Main_MyError_Failure = type {  }

%struct.Main_MySuccess = type { i32, [0 x i8] }

%struct.Main_MySuccess_Yay = type {  }

%struct.Main_Result_Err_Main_MySuccess__Main_MyError = type { %struct.Main_MyError }

%struct.Main_Result_Main_MySuccess__Main_MyError = type { i32, [4 x i8] }

%struct.Main_Result_Ok_Main_MySuccess__Main_MyError = type { %struct.Main_MySuccess }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_MyError = type { %struct.Main_MyError }

%struct.siko_Tuple_Main_MySuccess = type { %struct.Main_MySuccess }

define void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @siko_Tuple_Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MyError, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @siko_Tuple_Main_MySuccess(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MySuccess, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_6_1 = alloca %struct.siko_Tuple_, align 4
   %i_4_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_1 = alloca %struct.siko_Tuple_, align 4
   %i_0_3 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %a_0 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %i_0_1 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   call void @Main_someFunc(ptr %i_0_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %i_0_1, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_3, ptr align 4 %a_0, i8 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_1, i8 0, i1 false)
   call void @siko_Tuple_(ptr %i_1_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_2, i8 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Main_Result_Main_MySuccess__Main_MyError, ptr %i_0_3, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   %i_3_1 = bitcast %struct.Main_Result_Main_MySuccess__Main_MyError* %i_0_3 to %struct.siko_Tuple_Main_MyError*
   %i_3_2 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %i_3_1, i32 0, i32 0
   br label %block4
block4:
   call void @siko_Tuple_(ptr %i_4_1)
   br label %block1
block5:
   %i_5_1 = bitcast %struct.Main_Result_Main_MySuccess__Main_MyError* %i_0_3 to %struct.siko_Tuple_Main_MySuccess*
   %i_5_2 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %i_5_1, i32 0, i32 0
   br label %block6
block6:
   call void @siko_Tuple_(ptr %i_6_1)
   br label %block1
}

define void @Main_someFunc(ptr noundef %fn_result) {
block0:
   %i_0_2 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %i_0_1 = alloca %struct.Main_MyError, align 4
   call void @Main_MyError_Failure(ptr %i_0_1)
   call void @Main_Result_Err_Main_MySuccess__Main_MyError(ptr %i_0_1, ptr %i_0_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_2, i8 8, i1 false)
   ret void
}

define void @Main_MyError_Failure(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_MyError, align 4
   %tag = getelementptr inbounds %struct.Main_MyError, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_MyError, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_MyError_Failure*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @Main_Result_Err_Main_MySuccess__Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %tag = getelementptr inbounds %struct.Main_Result_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Result_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Main_Result_Err_Main_MySuccess__Main_MyError*
   %field0 = getelementptr inbounds %struct.Main_Result_Err_Main_MySuccess__Main_MyError, ptr %payload2, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


