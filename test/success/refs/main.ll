@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %unit_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %unit_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_1, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_25 = alloca %struct.siko_Tuple_, align 4
   %call_24 = alloca %struct.siko_Tuple_, align 4
   %ref_20 = alloca ptr, align 8
   %valueRef_19 = alloca %struct.String_String, align 8
   %call_18 = alloca %struct.siko_Tuple_, align 4
   %ref_17 = alloca ptr, align 8
   %valueRef_16 = alloca %struct.String_String, align 8
   %call_15 = alloca %struct.siko_Tuple_, align 4
   %implicitRef0 = alloca ptr, align 8
   %valueRef_14 = alloca %struct.String_String, align 8
   %ref2_13 = alloca ptr, align 8
   %valueRef_11 = alloca ptr, align 8
   %ref_10 = alloca ptr, align 8
   %ref_4 = alloca ptr, align 8
   %valueRef_3 = alloca %struct.String_String, align 8
   %s_2 = alloca %struct.String_String, align 8
   %literal_1 = alloca %struct.String_String, align 8
   %tmp_literal_1_1 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_1_1, align 8
   %tmp_literal_1_2 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 1
   store i64 3, ptr %tmp_literal_1_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %s_2, ptr align 8 %literal_1, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_3, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_3, ptr %ref_4, align 8
   store ptr %ref_4, ptr %ref_10, align 8
   store ptr %ref_10, ptr %valueRef_11, align 8
   store ptr %valueRef_11, ptr %ref2_13, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_14, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_14, ptr %implicitRef0, align 8
   %tmp_implicitRef0_0 = load ptr, ptr %implicitRef0, align 8
   call void @Main_foo(ptr %tmp_implicitRef0_0, ptr %call_15)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_16, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_16, ptr %ref_17, align 8
   %tmp_ref_17_0 = load ptr, ptr %ref_17, align 8
   call void @Main_foo(ptr %tmp_ref_17_0, ptr %call_18)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_19, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_19, ptr %ref_20, align 8
   %tmp_ref_20_0 = load ptr, ptr %ref_20, align 8
   call void @Main_foo(ptr %tmp_ref_20_0, ptr %call_24)
   call void @siko_Tuple_(ptr %unit_25)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_25, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


